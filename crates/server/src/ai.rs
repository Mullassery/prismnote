use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String, // "ollama", "claude", "openai"
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_model: Option<String>,
}

#[derive(Deserialize)]
pub struct AIRequest {
    pub action: String, // "explain", "fix", "complete"
    pub code: String,
    pub error: Option<String>,
    pub context: Option<String>,
}

#[derive(Serialize)]
pub struct AIResponse {
    pub suggestion: String,
    pub provider: String,
}

pub struct AIEngine {
    config: AIConfig,
}

impl AIEngine {
    pub fn new(config: AIConfig) -> Self {
        AIEngine { config }
    }

    pub async fn explain(&self, code: &str) -> Result<String> {
        match self.config.provider.as_str() {
            "ollama" => self.ollama_explain(code).await,
            "claude" => self.claude_explain(code).await,
            "openai" => self.openai_explain(code).await,
            _ => Err(anyhow!("Unknown AI provider")),
        }
    }

    pub async fn fix_error(&self, code: &str, error: &str) -> Result<String> {
        match self.config.provider.as_str() {
            "ollama" => self.ollama_fix(code, error).await,
            "claude" => self.claude_fix(code, error).await,
            "openai" => self.openai_fix(code, error).await,
            _ => Err(anyhow!("Unknown AI provider")),
        }
    }

    pub async fn complete_code(&self, code: &str, context: Option<&str>) -> Result<String> {
        match self.config.provider.as_str() {
            "ollama" => self.ollama_complete(code, context).await,
            "claude" => self.claude_complete(code, context).await,
            "openai" => self.openai_complete(code, context).await,
            _ => Err(anyhow!("Unknown AI provider")),
        }
    }

    // Ollama integration (local LLM)
    async fn ollama_explain(&self, code: &str) -> Result<String> {
        let ollama_url = self.config.ollama_url.as_ref().ok_or(anyhow!("Ollama URL not configured"))?;
        let model = self.config.ollama_model.as_ref().ok_or(anyhow!("Ollama model not selected"))?;

        let prompt = format!(
            "Explain this Python code briefly (2-3 sentences):\n\n```python\n{}\n```",
            code
        );

        let response = self.ollama_request(ollama_url, model, &prompt).await?;
        Ok(response)
    }

    async fn ollama_fix(&self, code: &str, error: &str) -> Result<String> {
        let ollama_url = self.config.ollama_url.as_ref().ok_or(anyhow!("Ollama URL not configured"))?;
        let model = self.config.ollama_model.as_ref().ok_or(anyhow!("Ollama model not selected"))?;

        let prompt = format!(
            "Fix this Python code that has an error:\n\nError: {}\n\nCode:\n```python\n{}\n```\n\nProvide corrected code only, no explanation.",
            error, code
        );

        let response = self.ollama_request(ollama_url, model, &prompt).await?;
        Ok(response)
    }

    async fn ollama_complete(&self, code: &str, context: Option<&str>) -> Result<String> {
        let ollama_url = self.config.ollama_url.as_ref().ok_or(anyhow!("Ollama URL not configured"))?;
        let model = self.config.ollama_model.as_ref().ok_or(anyhow!("Ollama model not selected"))?;

        let ctx = context.unwrap_or("");
        let prompt = format!(
            "Complete this Python code snippet. Only provide the completion, no explanation.\n\nContext: {}\n\n```python\n{}\n```\n\nCompletion:",
            ctx, code
        );

        let response = self.ollama_request(ollama_url, model, &prompt).await?;
        Ok(response)
    }

    async fn ollama_request(&self, url: &str, model: &str, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let body = json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        let response = client
            .post(format!("{}/api/generate", url))
            .json(&body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await?;

        let result: Value = response.json().await?;
        let response_text = result["response"]
            .as_str()
            .ok_or(anyhow!("No response from Ollama"))?
            .to_string();

        Ok(response_text.trim().to_string())
    }

    // Claude integration
    async fn claude_explain(&self, code: &str) -> Result<String> {
        let api_key = self.config.claude_api_key.as_ref().ok_or(anyhow!("Claude API key not configured"))?;

        let message = format!(
            "Explain this Python code briefly (2-3 sentences):\n\n```python\n{}\n```",
            code
        );

        self.claude_request(api_key, &message).await
    }

    async fn claude_fix(&self, code: &str, error: &str) -> Result<String> {
        let api_key = self.config.claude_api_key.as_ref().ok_or(anyhow!("Claude API key not configured"))?;

        let message = format!(
            "Fix this Python code that has an error:\n\nError: {}\n\nCode:\n```python\n{}\n```\n\nProvide corrected code only, no explanation.",
            error, code
        );

        self.claude_request(api_key, &message).await
    }

    async fn claude_complete(&self, code: &str, context: Option<&str>) -> Result<String> {
        let api_key = self.config.claude_api_key.as_ref().ok_or(anyhow!("Claude API key not configured"))?;

        let ctx = context.unwrap_or("");
        let message = format!(
            "Complete this Python code snippet. Only provide the completion, no explanation.\n\nContext: {}\n\n```python\n{}\n```\n\nCompletion:",
            ctx, code
        );

        self.claude_request(api_key, &message).await
    }

    async fn claude_request(&self, api_key: &str, message: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ]
        });

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Claude API error: {}", response.status()));
        }

        let result: Value = response.json().await?;
        let response_text = result["content"][0]["text"]
            .as_str()
            .ok_or(anyhow!("No response from Claude"))?
            .to_string();

        Ok(response_text)
    }

    // OpenAI integration
    async fn openai_explain(&self, code: &str) -> Result<String> {
        let api_key = self.config.openai_api_key.as_ref().ok_or(anyhow!("OpenAI API key not configured"))?;
        let model = self.config.openai_model.as_ref().ok_or(anyhow!("OpenAI model not selected"))?;

        let message = format!(
            "Explain this Python code briefly (2-3 sentences):\n\n```python\n{}\n```",
            code
        );

        self.openai_request(api_key, model, &message).await
    }

    async fn openai_fix(&self, code: &str, error: &str) -> Result<String> {
        let api_key = self.config.openai_api_key.as_ref().ok_or(anyhow!("OpenAI API key not configured"))?;
        let model = self.config.openai_model.as_ref().ok_or(anyhow!("OpenAI model not selected"))?;

        let message = format!(
            "Fix this Python code that has an error:\n\nError: {}\n\nCode:\n```python\n{}\n```\n\nProvide corrected code only.",
            error, code
        );

        self.openai_request(api_key, model, &message).await
    }

    async fn openai_complete(&self, code: &str, context: Option<&str>) -> Result<String> {
        let api_key = self.config.openai_api_key.as_ref().ok_or(anyhow!("OpenAI API key not configured"))?;
        let model = self.config.openai_model.as_ref().ok_or(anyhow!("OpenAI model not selected"))?;

        let ctx = context.unwrap_or("");
        let message = format!(
            "Complete this Python code. Only provide the completion.\n\nContext: {}\n\n```python\n{}\n```\n\nCompletion:",
            ctx, code
        );

        self.openai_request(api_key, model, &message).await
    }

    async fn openai_request(&self, api_key: &str, model: &str, message: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let body = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ],
            "max_tokens": 1024,
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("OpenAI API error: {}", response.status()));
        }

        let result: Value = response.json().await?;
        let response_text = result["choices"][0]["message"]["content"]
            .as_str()
            .ok_or(anyhow!("No response from OpenAI"))?
            .to_string();

        Ok(response_text)
    }
}
