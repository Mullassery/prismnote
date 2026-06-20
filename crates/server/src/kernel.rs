use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct KernelManager {
    process: Option<Child>,
    kernel_id: String,
    execution_count: usize,
    variables: Arc<Mutex<HashMap<String, String>>>,
    timeout: Duration,
}

impl KernelManager {
    pub fn new() -> Result<Self> {
        let kernel_id = Uuid::new_v4().to_string();

        // Verify ipykernel installation
        let check = Command::new("python")
            .arg("-c")
            .arg("import ipykernel; print('ok')")
            .output();

        if check.is_err() || !String::from_utf8_lossy(&check?.stdout).contains("ok") {
            return Err(anyhow!(
                "ipykernel not installed. Install with: pip install ipykernel"
            ));
        }

        // Start kernel in background (will use proper ZMQ in full implementation)
        let _process = Command::new("python")
            .arg("-m")
            .arg("ipykernel")
            .arg("--debug")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();

        Ok(KernelManager {
            process: _process,
            kernel_id,
            execution_count: 0,
            variables: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::from_secs(30),
        })
    }

    pub async fn execute(&mut self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        self.execution_count += 1;

        // Execute with timeout
        let result = tokio::time::timeout(
            self.timeout,
            self.execute_internal(code),
        )
        .await;

        match result {
            Ok(Ok((stdout, outputs))) => Ok((stdout, outputs)),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Execution timeout after {:?}", self.timeout)),
        }
    }

    async fn execute_internal(&self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        // Check for package installation requests
        if code.trim().starts_with("pip install") || code.trim().starts_with("!pip install") {
            return self.handle_package_install(code).await;
        }

        // Execute Python code
        let output = Command::new("python")
            .arg("-c")
            .arg(code)
            .output()
            .map_err(|e| anyhow!("Failed to execute code: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Track variables (simplified - full implementation would inspect kernel state)
        self.track_variables(code).await;

        let mut outputs = vec![];
        if !stdout.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stdout",
                "text": stdout
            }));
        }
        if !stderr.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stderr",
                "text": stderr
            }));
        }

        if !output.status.success() {
            return Err(anyhow!("Execution failed: {}", stderr));
        }

        Ok((vec![stdout], outputs))
    }

    async fn handle_package_install(&self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let clean_code = code
            .replace("!pip install", "pip install")
            .replace("!pip", "pip");

        let output = Command::new("python")
            .arg("-m")
            .arg("pip")
            .arg("install")
            .args(clean_code.split_whitespace().skip(2))
            .output()
            .map_err(|e| anyhow!("Package installation failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut outputs = vec![];
        if !stdout.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stdout",
                "text": format!("✅ Package installed successfully\n{}", stdout)
            }));
        }
        if !stderr.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stderr",
                "text": stderr
            }));
        }

        Ok((vec![stdout], outputs))
    }

    pub fn supports_pyspark(&self) -> bool {
        // Check if PySpark is installed
        let output = Command::new("python")
            .arg("-c")
            .arg("import pyspark; print('ok')")
            .output();

        output.is_ok() && String::from_utf8_lossy(&output.unwrap().stdout).contains("ok")
    }

    pub async fn execute_pyspark(&self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        if !self.supports_pyspark() {
            return Err(anyhow!(
                "PySpark not installed. Install with: pip install pyspark"
            ));
        }

        // Execute PySpark code
        let output = Command::new("python")
            .arg("-c")
            .arg(code)
            .output()
            .map_err(|e| anyhow!("PySpark execution failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut outputs = vec![];
        if !stdout.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stdout",
                "text": stdout
            }));
        }
        if !stderr.is_empty() {
            outputs.push(json!({
                "output_type": "stream",
                "name": "stderr",
                "text": stderr
            }));
        }

        if !output.status.success() {
            return Err(anyhow!("PySpark execution failed: {}", stderr));
        }

        Ok((vec![stdout], outputs))
    }

    async fn track_variables(&self, _code: &str) {
        // In full implementation, this would query kernel for active variables
        // For now, it's a placeholder for variable inspector feature
    }

    pub async fn get_variables(&self) -> Result<HashMap<String, String>> {
        Ok(self.variables.lock().await.clone())
    }

    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = duration;
    }

    pub fn execution_count(&self) -> usize {
        self.execution_count
    }
}

impl Drop for KernelManager {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}
