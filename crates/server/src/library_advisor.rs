use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibrarySuggestion {
    pub name: String,
    pub version: String,
    pub description: String,
    pub reasoning: String,
    pub installed_version: Option<String>,
    pub is_update: bool,
    pub category: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnoredLibrary {
    pub name: String,
    pub ignored_at: String,
}

pub struct LibraryAdvisor {
    ai_engine: Option<Arc<crate::ai::AIEngine>>,
}

impl LibraryAdvisor {
    pub fn new(ai_engine: Option<Arc<crate::ai::AIEngine>>) -> Self {
        LibraryAdvisor { ai_engine }
    }

    pub async fn suggest_libraries(
        &self,
        notebook_code: &str,
        installed_packages: Vec<String>,
        ignored_libraries: Vec<String>,
    ) -> Result<SuggestionsResponse> {
        // If no AI engine configured, return empty suggestions
        if self.ai_engine.is_none() {
            return Ok(SuggestionsResponse {
                suggestions: vec![],
                detected_intent: "Unknown (no AI configured)".to_string(),
                context_summary: "Configure an AI provider to get library suggestions".to_string(),
            });
        }

        let ai = self.ai_engine.as_ref().unwrap();

        let prompt = format!(
            "You are a Python library expert. Analyze this notebook code and suggest 3-5 libraries that would improve it.\n\n\
            Focus on:\n\
            1. Performance improvements\n\
            2. Code simplification\n\
            3. Missing best practices\n\
            4. Better alternatives\n\n\
            For each suggestion provide:\n\
            - name (exact PyPI package name)\n\
            - version (latest stable)\n\
            - description (one line)\n\
            - reasoning (why it helps THIS code specifically)\n\
            - category (data|viz|ml|web|utility)\n\
            - confidence (0-100)\n\n\
            Return ONLY valid JSON, no markdown:\n\
            {{\n\
              \"suggestions\": [\n\
                {{\n\
                  \"name\": \"library_name\",\n\
                  \"version\": \"X.Y.Z\",\n\
                  \"description\": \"...\",\n\
                  \"reasoning\": \"...\",\n\
                  \"category\": \"data\",\n\
                  \"confidence\": 95\n\
                }}\n\
              ],\n\
              \"detected_intent\": \"What is the code doing?\",\n\
              \"context_summary\": \"High-level summary\"\n\
            }}\n\n\
            Notebook code:\n{}\n\n\
            Already installed: {:?}\n\
            User ignored: {:?}",
            notebook_code, installed_packages, ignored_libraries
        );

        // Call Claude API
        let response_text = ai.call_api(&prompt).await?;

        // Parse JSON response
        let mut response: SuggestionsResponse = serde_json::from_str(&response_text)
            .unwrap_or(SuggestionsResponse {
                suggestions: vec![],
                detected_intent: "Analysis in progress".to_string(),
                context_summary: "Analyzing notebook patterns...".to_string(),
            });

        // Post-process: add installed version info and filter ignored
        for suggestion in &mut response.suggestions {
            // Check if already installed
            if let Some(installed) = installed_packages.iter().find(|p| p.starts_with(&suggestion.name)) {
                if let Some(version) = installed.split('=').nth(1) {
                    suggestion.installed_version = Some(version.to_string());
                    suggestion.is_update = version != &suggestion.version;
                }
            }
        }

        // Filter out ignored libraries
        response.suggestions.retain(|s| !ignored_libraries.contains(&s.name));

        Ok(response)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<LibrarySuggestion>,
    pub detected_intent: String,
    pub context_summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestLibrariesRequest {
    pub notebook_code: String,
    pub installed_packages: Vec<String>,
    pub ignored_libraries: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgnoreLibraryRequest {
    pub library_name: String,
    pub reason: Option<String>,
}
