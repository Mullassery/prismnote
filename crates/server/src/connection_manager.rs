use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConnectionType {
    #[serde(rename = "database")]
    Database,
    #[serde(rename = "data_warehouse")]
    DataWarehouse,
    #[serde(rename = "file_storage")]
    FileStorage,
    #[serde(rename = "duckdb")]
    DuckDB,
    #[serde(rename = "iceberg")]
    Iceberg,
    #[serde(rename = "api")]
    API,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConnectionStatus {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "connecting")]
    Connecting,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "unavailable")]
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalConnection {
    pub id: String,
    pub name: String,
    pub connection_type: String,
    pub provider: String, // postgres, mysql, s3, gcs, snowflake, bigquery, redshift, etc.
    pub status: String,
    pub connected_at: Option<String>,
    pub last_checked: String,
    pub latency_ms: Option<u32>,
    pub error_message: Option<String>,
    pub config: HashMap<String, String>,
    pub stats: ConnectionStats,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ConnectionStats {
    pub total_connections: u32,
    pub active_queries: u32,
    pub queries_run: u32,
    pub total_bytes_transferred: u64,
    pub uptime_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionsOverview {
    pub total_connections: usize,
    pub connected: usize,
    pub disconnected: usize,
    pub error_count: usize,
    pub connections: Vec<ExternalConnection>,
    pub last_refresh: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionHealthCheck {
    pub connection_id: String,
    pub is_healthy: bool,
    pub latency_ms: u32,
    pub timestamp: String,
    pub details: String,
}

pub struct ConnectionManager {
    connections: HashMap<String, ExternalConnection>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn register_connection(
        &mut self,
        name: String,
        connection_type: String,
        provider: String,
        config: HashMap<String, String>,
    ) -> Result<ExternalConnection, String> {
        let connection_id = Uuid::new_v4().to_string();
        let connection = ExternalConnection {
            id: connection_id.clone(),
            name,
            connection_type,
            provider,
            status: "disconnected".to_string(),
            connected_at: None,
            last_checked: Utc::now().to_rfc3339(),
            latency_ms: None,
            error_message: None,
            config,
            stats: ConnectionStats::default(),
        };

        self.connections.insert(connection_id.clone(), connection.clone());
        Ok(connection)
    }

    pub fn update_connection_status(
        &mut self,
        connection_id: &str,
        status: String,
        latency_ms: Option<u32>,
        error_message: Option<String>,
    ) -> Result<(), String> {
        if let Some(conn) = self.connections.get_mut(connection_id) {
            conn.status = status.clone();
            conn.last_checked = Utc::now().to_rfc3339();
            conn.latency_ms = latency_ms;
            conn.error_message = error_message;

            if status == "connected" {
                conn.connected_at = Some(Utc::now().to_rfc3339());
            }

            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub fn get_connection(&self, connection_id: &str) -> Option<ExternalConnection> {
        self.connections.get(connection_id).cloned()
    }

    pub fn get_all_connections(&self) -> Vec<ExternalConnection> {
        self.connections.values().cloned().collect()
    }

    pub fn get_connections_overview(&self) -> ConnectionsOverview {
        let connections = self.get_all_connections();
        let connected = connections.iter().filter(|c| c.status == "connected").count();
        let disconnected = connections.iter().filter(|c| c.status == "disconnected").count();
        let error_count = connections.iter().filter(|c| c.status == "error").count();

        ConnectionsOverview {
            total_connections: connections.len(),
            connected,
            disconnected,
            error_count,
            connections,
            last_refresh: Utc::now().to_rfc3339(),
        }
    }

    pub fn remove_connection(&mut self, connection_id: &str) -> Result<(), String> {
        if self.connections.remove(connection_id).is_some() {
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn check_connection_health(
        &self,
        connection_id: &str,
    ) -> Result<ConnectionHealthCheck, String> {
        if let Some(conn) = self.get_connection(connection_id) {
            // In production, this would actually test the connection
            // For now, return the last known status
            Ok(ConnectionHealthCheck {
                connection_id: connection_id.to_string(),
                is_healthy: conn.status == "connected",
                latency_ms: conn.latency_ms.unwrap_or(0),
                timestamp: Utc::now().to_rfc3339(),
                details: format!("Connection status: {}", conn.status),
            })
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub fn update_connection_stats(
        &mut self,
        connection_id: &str,
        stats: ConnectionStats,
    ) -> Result<(), String> {
        if let Some(conn) = self.connections.get_mut(connection_id) {
            conn.stats = stats;
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub fn get_connections_by_type(&self, conn_type: &str) -> Vec<ExternalConnection> {
        self.connections
            .values()
            .filter(|c| c.connection_type == conn_type)
            .cloned()
            .collect()
    }
}
