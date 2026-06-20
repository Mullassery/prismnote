use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::process::{Child, Command};
use std::time::Duration;
use uuid::Uuid;

pub struct KernelManager {
    process: Option<Child>,
    kernel_id: String,
    execution_count: usize,
}

impl KernelManager {
    pub fn new() -> Result<Self> {
        let kernel_id = Uuid::new_v4().to_string();

        // Try to detect ipykernel installation
        let check = Command::new("python")
            .arg("-c")
            .arg("import ipykernel; print('ok')")
            .output();

        if check.is_err() || !String::from_utf8_lossy(&check?.stdout).contains("ok") {
            return Err(anyhow!(
                "ipykernel not installed. Install with: pip install ipykernel"
            ));
        }

        // Start kernel in background
        let process = Command::new("python")
            .arg("-m")
            .arg("ipykernel")
            .arg("--debug")
            .spawn()
            .ok();

        Ok(KernelManager {
            process,
            kernel_id,
            execution_count: 0,
        })
    }

    pub async fn execute(&mut self, code: &str) -> Result<(Vec<String>, Vec<Value>)> {
        self.execution_count += 1;

        // Fallback: execute Python locally (until ZMQ is fully integrated)
        // In production, this would send ZMQ messages to the kernel

        let output = Command::new("python")
            .arg("-c")
            .arg(code)
            .output()
            .map_err(|e| anyhow!("Failed to execute code: {}", e))?;

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
            return Err(anyhow!("Execution failed: {}", stderr));
        }

        Ok((vec![stdout], outputs))
    }
}

impl Drop for KernelManager {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}
