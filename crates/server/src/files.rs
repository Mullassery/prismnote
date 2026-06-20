use crate::models::{Cell, Notebook, Output};
use anyhow::Result;
use serde_json::{json, Value};

/// Converts a Jupyter .ipynb notebook to our internal Notebook format
pub fn from_ipynb(ipynb: Value) -> Result<Notebook> {
    let cells: Vec<Cell> = ipynb["cells"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|cell| {
            let cell_type = cell["cell_type"].as_str().unwrap_or("code").to_string();
            let source = cell["source"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            let outputs: Vec<Output> = cell["outputs"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|out| Output {
                    output_type: out["output_type"].as_str().unwrap_or("stream").to_string(),
                    data: out["data"].as_object().map(|_| out["data"].clone()),
                    text: out["text"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|s| s.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        }),
                    metadata: out["metadata"].as_object().map(|_| out["metadata"].clone()),
                })
                .collect();

            Cell {
                id: uuid::Uuid::new_v4().to_string(),
                cell_type,
                source,
                outputs,
                execution_count: cell["execution_count"].as_u64().map(|n| n as usize),
                metadata: cell["metadata"].clone(),
            }
        })
        .collect();

    Ok(Notebook {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Imported Notebook".to_string(),
        cells,
        metadata: ipynb.get("metadata").cloned().unwrap_or(json!({})),
    })
}

/// Converts our internal Notebook to .ipynb format
pub fn to_ipynb(notebook: &Notebook) -> Value {
    json!({
        "cells": notebook.cells.iter().map(|cell| {
            json!({
                "cell_type": cell.cell_type,
                "execution_count": cell.execution_count,
                "metadata": cell.metadata,
                "outputs": cell.outputs.iter().map(|out| {
                    let mut output = json!({
                        "output_type": out.output_type,
                    });
                    if let Some(data) = &out.data {
                        output["data"] = data.clone();
                    }
                    if let Some(text) = &out.text {
                        output["text"] = json!(text);
                    }
                    if let Some(meta) = &out.metadata {
                        output["metadata"] = meta.clone();
                    }
                    output
                }).collect::<Vec<_>>(),
                "source": cell.source,
            })
        }).collect::<Vec<_>>(),
        "metadata": notebook.metadata,
        "nbformat": 4,
        "nbformat_minor": 5,
    })
}
