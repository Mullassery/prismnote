use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SQLQuery {
    pub query: String,
    pub connection_id: String,
    pub timeout_seconds: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub estimated_memory_bytes: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryOptimization {
    pub issue: String,
    pub severity: String, // "low", "medium", "high"
    pub suggestion: String,
    pub estimated_impact: String, // e.g., "10-50% faster"
}

pub struct SQLExecutor;

impl SQLExecutor {
    pub fn parse_sql_cell(code: &str) -> Option<String> {
        let trimmed = code.trim();

        // Check for --sql marker
        if trimmed.starts_with("--sql") {
            return Some(
                trimmed
                    .strip_prefix("--sql")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        }

        // Check for %sql magic
        if trimmed.starts_with("%sql") {
            return Some(
                trimmed
                    .strip_prefix("%sql")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        }

        None
    }

    pub fn is_sql_cell(code: &str) -> bool {
        Self::parse_sql_cell(code).is_some()
    }

    pub async fn execute_query(
        _query: &str,
        _connection_id: &str,
    ) -> Result<QueryResult> {
        // TODO: Integrate with database manager
        // For now, return placeholder result
        Ok(QueryResult {
            columns: vec!["id".to_string(), "value".to_string()],
            rows: vec![
                vec![json!(1), json!("test")],
                vec![json!(2), json!("data")],
            ],
            row_count: 2,
            execution_time_ms: 150,
            estimated_memory_bytes: 2048,
        })
    }

    pub fn format_result_as_html(result: &QueryResult) -> String {
        let mut html = String::from("<table border='1' cellpadding='5' cellspacing='0'>");

        // Header
        html.push_str("<thead><tr>");
        for col in &result.columns {
            html.push_str(&format!("<th>{}</th>", escape_html(col)));
        }
        html.push_str("</tr></thead>");

        // Rows (limit to 1000 for display)
        html.push_str("<tbody>");
        for (idx, row) in result.rows.iter().enumerate() {
            if idx >= 1000 {
                html.push_str(&format!(
                    "<tr><td colspan='{}'><i>... {} more rows</i></td></tr>",
                    result.columns.len(),
                    result.row_count - 1000
                ));
                break;
            }

            html.push_str("<tr>");
            for cell in row {
                let cell_str = match cell {
                    Value::Null => "NULL".to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => escape_html(s),
                    Value::Array(_) | Value::Object(_) => cell.to_string(),
                };
                html.push_str(&format!("<td>{}</td>", cell_str));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody>");
        html.push_str("</table>");

        // Summary
        html.push_str(&format!(
            "<p><i>Rows: {}, Execution time: {}ms</i></p>",
            result.row_count, result.execution_time_ms
        ));

        html
    }

    pub fn analyze_query(query: &str) -> Vec<QueryOptimization> {
        let mut optimizations = vec![];
        let upper = query.to_uppercase();

        // Check for SELECT *
        if upper.contains("SELECT *") {
            optimizations.push(QueryOptimization {
                issue: "SELECT * is used".to_string(),
                severity: "medium".to_string(),
                suggestion: "Specify only needed columns to reduce data transfer and improve performance".to_string(),
                estimated_impact: "5-20% faster".to_string(),
            });
        }

        // Check for missing WHERE clause on large table
        if upper.contains("FROM") && !upper.contains("WHERE") {
            optimizations.push(QueryOptimization {
                issue: "No WHERE clause detected".to_string(),
                severity: "high".to_string(),
                suggestion: "Add WHERE clause to filter results and reduce data scanned".to_string(),
                estimated_impact: "50-90% faster".to_string(),
            });
        }

        // Check for LIKE without index-friendly patterns
        if upper.contains("LIKE") && upper.contains("LIKE '%") {
            optimizations.push(QueryOptimization {
                issue: "LIKE with leading wildcard detected".to_string(),
                severity: "high".to_string(),
                suggestion: "Use exact matches or indexed LIKE patterns starting with literal characters".to_string(),
                estimated_impact: "10-100x faster".to_string(),
            });
        }

        // Check for missing JOINs (subqueries where JOIN would be better)
        if upper.matches("SELECT").count() > 1 {
            optimizations.push(QueryOptimization {
                issue: "Nested SELECT (subquery) detected".to_string(),
                severity: "medium".to_string(),
                suggestion: "Consider using JOIN instead of subquery for better performance".to_string(),
                estimated_impact: "5-50% faster".to_string(),
            });
        }

        // Check for NOT IN with subquery
        if upper.contains("NOT IN") && upper.contains("SELECT") {
            optimizations.push(QueryOptimization {
                issue: "NOT IN with subquery detected".to_string(),
                severity: "medium".to_string(),
                suggestion: "Use NOT EXISTS (more efficient) or LEFT JOIN instead of NOT IN".to_string(),
                estimated_impact: "10-50% faster".to_string(),
            });
        }

        // Check for OR conditions (often slow)
        if upper.matches(" OR ").count() > 2 {
            optimizations.push(QueryOptimization {
                issue: "Multiple OR conditions detected".to_string(),
                severity: "low".to_string(),
                suggestion: "Consider using IN clause or UNION if appropriate for better optimization".to_string(),
                estimated_impact: "5-30% faster".to_string(),
            });
        }

        // Check for functions on WHERE columns
        if upper.contains("WHERE") && (upper.contains("LOWER(") || upper.contains("UPPER(")) {
            optimizations.push(QueryOptimization {
                issue: "Function applied in WHERE clause".to_string(),
                severity: "medium".to_string(),
                suggestion: "Store data in normalized form or use case-insensitive collation instead".to_string(),
                estimated_impact: "10-50% faster".to_string(),
            });
        }

        optimizations
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_detection() {
        assert!(SQLExecutor::is_sql_cell("--sql SELECT * FROM users"));
        assert!(SQLExecutor::is_sql_cell("%sql SELECT * FROM users"));
        assert!(!SQLExecutor::is_sql_cell("SELECT * FROM users"));
    }

    #[test]
    fn test_sql_parsing() {
        let sql = SQLExecutor::parse_sql_cell("--sql SELECT * FROM users WHERE id = 1");
        assert_eq!(sql, Some("SELECT * FROM users WHERE id = 1".to_string()));
    }

    #[test]
    fn test_query_analysis() {
        let optimizations = SQLExecutor::analyze_query("SELECT * FROM users");
        assert!(!optimizations.is_empty());
        assert!(optimizations
            .iter()
            .any(|o| o.issue.contains("SELECT *")));
    }
}
