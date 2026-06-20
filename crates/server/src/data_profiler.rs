use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnProfile {
    pub name: String,
    pub data_type: String,
    pub non_null_count: usize,
    pub null_count: usize,
    pub unique_count: usize,
    pub stats: ColumnStats,
    pub sample_values: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnStats {
    pub min: Option<String>,
    pub max: Option<String>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std_dev: Option<f64>,
    pub percentile_25: Option<f64>,
    pub percentile_75: Option<f64>,
    pub quartile_1: Option<f64>,
    pub quartile_3: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataFrameProfile {
    pub name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub memory_usage_bytes: usize,
    pub columns: Vec<ColumnProfile>,
    pub missing_data_pattern: MissingDataPattern,
    pub correlation_matrix: Option<Vec<Vec<f64>>>,
    pub profile_timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissingDataPattern {
    pub total_missing: usize,
    pub missing_percentage: f64,
    pub columns_with_missing: Vec<(String, usize, f64)>, // (column_name, count, percentage)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataQualityIssue {
    pub column: String,
    pub issue_type: String, // "missing_data", "outlier", "invalid_type", "duplication"
    pub severity: String,   // "low", "medium", "high"
    pub description: String,
    pub suggestion: String,
}

pub struct DataProfiler;

impl DataProfiler {
    pub fn profile_dataframe(
        dataframe_json: &serde_json::Value,
        name: &str,
    ) -> Result<DataFrameProfile> {
        let rows = dataframe_json.as_array().ok_or(anyhow::anyhow!("Expected array"))?;
        let row_count = rows.len();

        if row_count == 0 {
            return Ok(DataFrameProfile {
                name: name.to_string(),
                row_count: 0,
                column_count: 0,
                memory_usage_bytes: 0,
                columns: vec![],
                missing_data_pattern: MissingDataPattern {
                    total_missing: 0,
                    missing_percentage: 0.0,
                    columns_with_missing: vec![],
                },
                correlation_matrix: None,
                profile_timestamp: chrono::Local::now().to_rfc3339(),
            });
        }

        // Extract column names from first row
        let first_row = &rows[0];
        let columns: Vec<String> = first_row
            .as_object()
            .ok_or(anyhow::anyhow!("Expected object"))?
            .keys()
            .cloned()
            .collect();

        let column_count = columns.len();
        let mut column_profiles = vec![];
        let mut total_missing = 0;

        for col_name in &columns {
            let mut values = vec![];
            let mut null_count = 0;
            let mut sample_values = vec![];

            for row in rows {
                if let Some(obj) = row.as_object() {
                    if let Some(val) = obj.get(col_name) {
                        if val.is_null() {
                            null_count += 1;
                        } else {
                            let str_val = val.to_string();
                            values.push(str_val.clone());
                            if sample_values.len() < 5 {
                                sample_values.push(str_val);
                            }
                        }
                    } else {
                        null_count += 1;
                    }
                }
            }

            let non_null_count = row_count - null_count;
            let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();

            total_missing += null_count;

            let stats = Self::calculate_stats(&values);
            let data_type = Self::infer_type(&values);

            column_profiles.push(ColumnProfile {
                name: col_name.clone(),
                data_type,
                non_null_count,
                null_count,
                unique_count,
                stats,
                sample_values,
            });
        }

        let missing_percentage = if row_count > 0 {
            (total_missing as f64 / (row_count as f64 * column_count as f64)) * 100.0
        } else {
            0.0
        };

        let columns_with_missing: Vec<(String, usize, f64)> = column_profiles
            .iter()
            .filter(|c| c.null_count > 0)
            .map(|c| {
                let percentage = (c.null_count as f64 / row_count as f64) * 100.0;
                (c.name.clone(), c.null_count, percentage)
            })
            .collect();

        Ok(DataFrameProfile {
            name: name.to_string(),
            row_count,
            column_count,
            memory_usage_bytes: Self::estimate_memory(&column_profiles, row_count),
            columns: column_profiles,
            missing_data_pattern: MissingDataPattern {
                total_missing,
                missing_percentage,
                columns_with_missing,
            },
            correlation_matrix: None,
            profile_timestamp: chrono::Local::now().to_rfc3339(),
        })
    }

    pub fn detect_quality_issues(profile: &DataFrameProfile) -> Vec<DataQualityIssue> {
        let mut issues = vec![];

        for col in &profile.columns {
            // Check for high missing data
            if col.null_count > profile.row_count / 2 {
                issues.push(DataQualityIssue {
                    column: col.name.clone(),
                    issue_type: "missing_data".to_string(),
                    severity: "high".to_string(),
                    description: format!(
                        "{:.1}% of values are missing",
                        (col.null_count as f64 / profile.row_count as f64) * 100.0
                    ),
                    suggestion: "Consider dropping this column or imputing missing values".to_string(),
                });
            }

            // Check for low variance (mostly duplicates)
            if col.unique_count < profile.row_count / 100 && profile.row_count > 100 {
                issues.push(DataQualityIssue {
                    column: col.name.clone(),
                    issue_type: "low_variance".to_string(),
                    severity: "medium".to_string(),
                    description: format!(
                        "Only {}/{} unique values",
                        col.unique_count, profile.row_count
                    ),
                    suggestion: "This column may have low predictive power".to_string(),
                });
            }

            // Check for outliers (if numeric)
            if col.data_type == "numeric" {
                if let Some(std) = col.stats.std_dev {
                    if let Some(mean) = col.stats.mean {
                        if std > mean * 2.0 {
                            issues.push(DataQualityIssue {
                                column: col.name.clone(),
                                issue_type: "outlier".to_string(),
                                severity: "medium".to_string(),
                                description: "High standard deviation suggests outliers".to_string(),
                                suggestion: "Consider using robust scaling or outlier removal"
                                    .to_string(),
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    fn calculate_stats(values: &[String]) -> ColumnStats {
        let mut numeric_values: Vec<f64> = values
            .iter()
            .filter_map(|v| v.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return ColumnStats {
                min: None,
                max: None,
                mean: None,
                median: None,
                std_dev: None,
                percentile_25: None,
                percentile_75: None,
                quartile_1: None,
                quartile_3: None,
            };
        }

        numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = numeric_values.first().copied();
        let max = numeric_values.last().copied();
        let mean = if numeric_values.is_empty() {
            None
        } else {
            Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64)
        };

        let median = if numeric_values.len() % 2 == 0 {
            let mid = numeric_values.len() / 2;
            Some((numeric_values[mid - 1] + numeric_values[mid]) / 2.0)
        } else {
            Some(numeric_values[numeric_values.len() / 2])
        };

        let std_dev = if let Some(m) = mean {
            let variance = numeric_values
                .iter()
                .map(|v| (v - m).powi(2))
                .sum::<f64>()
                / numeric_values.len() as f64;
            Some(variance.sqrt())
        } else {
            None
        };

        ColumnStats {
            min: min.map(|v| v.to_string()),
            max: max.map(|v| v.to_string()),
            mean,
            median,
            std_dev,
            percentile_25: Some(numeric_values[(numeric_values.len() / 4).max(0)]),
            percentile_75: Some(numeric_values[((numeric_values.len() * 3) / 4).min(numeric_values.len() - 1)]),
            quartile_1: Some(numeric_values[(numeric_values.len() / 4).max(0)]),
            quartile_3: Some(numeric_values[((numeric_values.len() * 3) / 4).min(numeric_values.len() - 1)]),
        }
    }

    fn infer_type(values: &[String]) -> String {
        if values.is_empty() {
            return "unknown".to_string();
        }

        let numeric_count = values
            .iter()
            .filter(|v| v.parse::<f64>().is_ok())
            .count();

        let bool_count = values
            .iter()
            .filter(|v| v.to_lowercase() == "true" || v.to_lowercase() == "false")
            .count();

        if numeric_count == values.len() {
            "numeric".to_string()
        } else if bool_count == values.len() {
            "boolean".to_string()
        } else {
            "string".to_string()
        }
    }

    fn estimate_memory(columns: &[ColumnProfile], row_count: usize) -> usize {
        let mut total = 0;
        for col in columns {
            let bytes_per_value = match col.data_type.as_str() {
                "numeric" => 8,
                "boolean" => 1,
                _ => col.sample_values.iter().map(|v| v.len()).max().unwrap_or(50),
            };
            total += bytes_per_value * row_count;
        }
        total
    }
}
