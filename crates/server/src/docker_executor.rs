use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub port_bindings: HashMap<String, String>,
    pub environment: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainerExecution {
    pub container_id: String,
    pub command: String,
    pub working_dir: String,
    pub environment: HashMap<String, String>,
    pub timeout_seconds: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub container_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainerFile {
    pub container_id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified_time: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerStats {
    pub container_id: String,
    pub cpu_percent: f32,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub block_read: u64,
    pub block_write: u64,
}

pub struct DockerExecutor {
    pub docker_host: String,
    pub timeout_seconds: u32,
}

impl DockerExecutor {
    pub fn new(docker_host: Option<String>) -> Self {
        Self {
            docker_host: docker_host
                .unwrap_or_else(|| "unix:///var/run/docker.sock".to_string()),
            timeout_seconds: 30,
        }
    }

    pub async fn list_containers(&self) -> Result<Vec<DockerContainer>, String> {
        // TODO: Use docker API or CLI to list containers
        Ok(vec![
            DockerContainer {
                id: "abc123def456".to_string(),
                name: "prismnote-dev".to_string(),
                image: "python:3.11".to_string(),
                status: "running".to_string(),
                port_bindings: HashMap::from([
                    ("8000/tcp".to_string(), "8000".to_string()),
                ]),
                environment: vec![
                    "PYTHONUNBUFFERED=1".to_string(),
                    "PRISMNOTE_DEV=1".to_string(),
                ],
            },
        ])
    }

    pub async fn execute_in_container(
        &self,
        container_id: &str,
        command: &str,
        working_dir: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let start = std::time::Instant::now();

        // TODO: Implement actual Docker container code execution
        // Would use docker exec command or Docker API
        // docker exec -w /workspace <container_id> <command>

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            container_id: container_id.to_string(),
            exit_code: 0,
            stdout: format!("Executed in container: {}", command),
            stderr: String::new(),
            execution_time_ms,
        })
    }

    pub async fn get_container_files(
        &self,
        container_id: &str,
        path: &str,
    ) -> Result<Vec<ContainerFile>, String> {
        // TODO: List files in container directory
        Ok(vec![
            ContainerFile {
                container_id: container_id.to_string(),
                path: path.to_string(),
                name: "example.py".to_string(),
                size: 1024,
                is_dir: false,
                modified_time: chrono::Local::now().to_rfc3339(),
            },
        ])
    }

    pub async fn read_container_file(
        &self,
        container_id: &str,
        path: &str,
    ) -> Result<String, String> {
        // TODO: Read file content from container
        Ok(format!(
            "# File from container {}: {}\nprint('Hello from container')",
            container_id, path
        ))
    }

    pub async fn write_container_file(
        &self,
        container_id: &str,
        path: &str,
        content: String,
    ) -> Result<(), String> {
        // TODO: Write file to container
        Ok(())
    }

    pub async fn create_container(
        &self,
        image: &str,
        name: &str,
        env: HashMap<String, String>,
        ports: HashMap<String, u16>,
    ) -> Result<DockerContainer, String> {
        // TODO: Create new Docker container
        let mut port_bindings = HashMap::new();
        for (internal, external) in ports {
            port_bindings.insert(format!("{}/tcp", internal), external.to_string());
        }

        Ok(DockerContainer {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            image: image.to_string(),
            status: "created".to_string(),
            port_bindings,
            environment: env.iter().map(|(k, v)| format!("{}={}", k, v)).collect(),
        })
    }

    pub async fn start_container(&self, container_id: &str) -> Result<(), String> {
        // TODO: Start container using Docker API
        Ok(())
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<(), String> {
        // TODO: Stop container
        Ok(())
    }

    pub async fn remove_container(&self, container_id: &str) -> Result<(), String> {
        // TODO: Remove container
        Ok(())
    }

    pub async fn get_container_stats(&self, container_id: &str) -> Result<DockerStats, String> {
        // TODO: Get container resource usage stats
        Ok(DockerStats {
            container_id: container_id.to_string(),
            cpu_percent: 25.5,
            memory_usage: 512 * 1024 * 1024,
            memory_limit: 2 * 1024 * 1024 * 1024,
            network_rx: 1024 * 1024,
            network_tx: 512 * 1024,
            block_read: 10 * 1024 * 1024,
            block_write: 5 * 1024 * 1024,
        })
    }

    pub async fn pull_image(&self, image: &str) -> Result<String, String> {
        // TODO: Pull Docker image from registry
        Ok(format!("Pulled image: {}", image))
    }

    pub async fn get_container_logs(
        &self,
        container_id: &str,
        lines: Option<u32>,
    ) -> Result<String, String> {
        // TODO: Get container logs
        let num_lines = lines.unwrap_or(50);
        Ok(format!("Last {} lines from container {}", num_lines, container_id))
    }

    pub async fn copy_to_container(
        &self,
        container_id: &str,
        local_path: &str,
        container_path: &str,
    ) -> Result<(), String> {
        // TODO: Copy file from host to container
        Ok(())
    }

    pub async fn copy_from_container(
        &self,
        container_id: &str,
        container_path: &str,
        local_path: &str,
    ) -> Result<(), String> {
        // TODO: Copy file from container to host
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PythonContainerCell {
    pub cell_id: String,
    pub container_id: String,
    pub code: String,
    pub working_dir: String,
}

pub struct ContainerCellExecutor;

impl ContainerCellExecutor {
    pub async fn execute_python_in_container(
        container_id: &str,
        code: &str,
        working_dir: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let executor = DockerExecutor::new(None);
        let wd = working_dir.unwrap_or("/workspace".to_string());

        executor
            .execute_in_container(container_id, &format!("python -c '{}'", code), Some(wd))
            .await
    }

    pub async fn execute_shell_in_container(
        container_id: &str,
        command: &str,
        working_dir: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let executor = DockerExecutor::new(None);
        let wd = working_dir.unwrap_or("/".to_string());

        executor
            .execute_in_container(container_id, command, Some(wd))
            .await
    }

    pub async fn execute_file_in_container(
        container_id: &str,
        file_path: &str,
        working_dir: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let executor = DockerExecutor::new(None);
        let wd = working_dir.unwrap_or("/workspace".to_string());

        executor
            .execute_in_container(container_id, &format!("python {}", file_path), Some(wd))
            .await
    }
}
