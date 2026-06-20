use anyhow::Result;
use serde_json::Value;
use std::time::Duration;

pub struct CellExecutor {
    timeout: Duration,
    max_output_size: usize,
}

impl CellExecutor {
    pub fn new() -> Self {
        CellExecutor {
            timeout: Duration::from_secs(30),
            max_output_size: 1024 * 1024 * 10, // 10 MB
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn execute_with_timeout(&self, code: &str) -> Result<(String, Vec<Value>)> {
        // Validate code doesn't exceed limits
        if code.len() > 1024 * 1024 {
            anyhow::bail!("Code exceeds maximum size (1 MB)");
        }

        // Execute with timeout
        let result = tokio::time::timeout(
            self.timeout,
            self.execute_code(code),
        )
        .await;

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!(
                "Cell execution timeout (exceeded {:?})",
                self.timeout
            )),
        }
    }

    async fn execute_code(&self, code: &str) -> Result<(String, Vec<Value>)> {
        // Placeholder for actual execution logic
        // This would be called by the kernel manager

        // Validate code
        if code.trim().is_empty() {
            return Ok((String::new(), vec![]));
        }

        // Check for dangerous operations
        self.validate_code(code)?;

        Ok((String::new(), vec![]))
    }

    fn validate_code(&self, code: &str) -> Result<()> {
        // Check for potentially dangerous operations
        let dangerous_patterns = vec![
            "os.system",
            "subprocess.call",
            "exec(",
            "eval(",
            "__import__",
        ];

        for pattern in dangerous_patterns {
            if code.contains(pattern) {
                // Don't block, just warn (these are legitimate in notebooks)
                tracing::warn!("Potentially unsafe operation detected: {}", pattern);
            }
        }

        Ok(())
    }

    pub fn get_timeout(&self) -> Duration {
        self.timeout
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn set_max_output_size(&mut self, size: usize) {
        self.max_output_size = size;
    }

    pub fn truncate_output(&self, output: &str) -> String {
        if output.len() > self.max_output_size {
            format!(
                "{}...\n\n[Output truncated - exceeded {} bytes]",
                &output[..self.max_output_size],
                self.max_output_size
            )
        } else {
            output.to_string()
        }
    }
}
