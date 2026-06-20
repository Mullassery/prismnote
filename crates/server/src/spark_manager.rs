use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparkConfig {
    pub app_name: String,
    pub master: String, // "local[*]", "local[4]", "spark://host:7077"
    pub executor_memory: String, // "2g", "4g", "8g"
    pub driver_memory: String,
    pub executor_cores: u32,
    pub executor_instances: u32,
    pub shuffle_partitions: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparkSessionInfo {
    pub app_id: String,
    pub app_name: String,
    pub master: String,
    pub is_running: bool,
    pub created_at: String,
    pub uptime_seconds: u64,
    pub executor_memory: String,
    pub driver_memory: String,
    pub total_cores: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataFrameInfo {
    pub name: String,
    pub row_count: u64,
    pub column_count: usize,
    pub columns: Vec<ColumnInfo>,
    pub size_bytes: u64,
    pub cached: bool,
    pub partitions: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DAGStage {
    pub stage_id: u32,
    pub stage_name: String,
    pub num_tasks: u32,
    pub parent_stages: Vec<u32>,
    pub rdd_ids: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionDAG {
    pub job_id: u32,
    pub stages: Vec<DAGStage>,
    pub execution_time_ms: u64,
}

pub struct SparkManager {
    sessions: HashMap<String, SparkSessionInfo>,
    dataframes: HashMap<String, DataFrameInfo>,
    dags: HashMap<u32, ExecutionDAG>,
}

impl SparkManager {
    pub fn new() -> Self {
        SparkManager {
            sessions: HashMap::new(),
            dataframes: HashMap::new(),
            dags: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, config: SparkConfig) -> Result<SparkSessionInfo> {
        let app_id = format!("app-{}", uuid::Uuid::new_v4());

        let session_info = SparkSessionInfo {
            app_id: app_id.clone(),
            app_name: config.app_name.clone(),
            master: config.master.clone(),
            is_running: true,
            created_at: chrono::Local::now().to_rfc3339(),
            uptime_seconds: 0,
            executor_memory: config.executor_memory.clone(),
            driver_memory: config.driver_memory.clone(),
            total_cores: config.executor_cores * config.executor_instances,
        };

        self.sessions.insert(app_id.clone(), session_info.clone());
        Ok(session_info)
    }

    pub fn get_session(&self, app_id: &str) -> Option<SparkSessionInfo> {
        self.sessions.get(app_id).cloned()
    }

    pub fn list_sessions(&self) -> Vec<SparkSessionInfo> {
        self.sessions.values().cloned().collect()
    }

    pub fn stop_session(&mut self, app_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(app_id) {
            session.is_running = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", app_id))
        }
    }

    pub fn register_dataframe(&mut self, name: String, info: DataFrameInfo) {
        self.dataframes.insert(name, info);
    }

    pub fn get_dataframe(&self, name: &str) -> Option<DataFrameInfo> {
        self.dataframes.get(name).cloned()
    }

    pub fn list_dataframes(&self) -> Vec<DataFrameInfo> {
        self.dataframes.values().cloned().collect()
    }

    pub fn cache_dataframe(&mut self, name: &str) -> Result<()> {
        if let Some(df) = self.dataframes.get_mut(name) {
            df.cached = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("DataFrame not found: {}", name))
        }
    }

    pub fn record_dag(&mut self, job_id: u32, dag: ExecutionDAG) {
        self.dags.insert(job_id, dag);
    }

    pub fn get_dag(&self, job_id: u32) -> Option<ExecutionDAG> {
        self.dags.get(&job_id).cloned()
    }

    pub fn analyze_shuffle(&self, df_name: &str) -> ShuffleAnalysis {
        if let Some(df) = self.dataframes.get(df_name) {
            ShuffleAnalysis {
                dataframe_name: df_name.to_string(),
                row_count: df.row_count,
                partitions: df.partitions,
                estimated_data_per_partition_mb: (df.size_bytes / (df.partitions as u64 + 1)) / 1024 / 1024,
                shuffle_risk: if df.row_count > 1_000_000 && df.partitions < 16 {
                    "high".to_string()
                } else if df.row_count > 100_000 && df.partitions < 8 {
                    "medium".to_string()
                } else {
                    "low".to_string()
                },
            }
        } else {
            ShuffleAnalysis {
                dataframe_name: df_name.to_string(),
                row_count: 0,
                partitions: 0,
                estimated_data_per_partition_mb: 0,
                shuffle_risk: "unknown".to_string(),
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShuffleAnalysis {
    pub dataframe_name: String,
    pub row_count: u64,
    pub partitions: u32,
    pub estimated_data_per_partition_mb: u64,
    pub shuffle_risk: String,
}

impl Default for SparkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = SparkManager::new();
        let config = SparkConfig {
            app_name: "test-app".to_string(),
            master: "local[*]".to_string(),
            executor_memory: "2g".to_string(),
            driver_memory: "1g".to_string(),
            executor_cores: 4,
            executor_instances: 1,
            shuffle_partitions: 200,
        };

        let session = manager.create_session(config).unwrap();
        assert!(session.is_running);
        assert_eq!(session.total_cores, 4);
    }

    #[test]
    fn test_session_lifecycle() {
        let mut manager = SparkManager::new();
        let config = SparkConfig {
            app_name: "test-app".to_string(),
            master: "local[*]".to_string(),
            executor_memory: "2g".to_string(),
            driver_memory: "1g".to_string(),
            executor_cores: 4,
            executor_instances: 1,
            shuffle_partitions: 200,
        };

        let session = manager.create_session(config).unwrap();
        assert!(manager.get_session(&session.app_id).is_some());

        manager.stop_session(&session.app_id).unwrap();
        let stopped = manager.get_session(&session.app_id).unwrap();
        assert!(!stopped.is_running);
    }
}
