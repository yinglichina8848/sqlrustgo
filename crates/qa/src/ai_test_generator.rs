//! AI Test Generator
//!
//! Generates test cases using LLM APIs for SQLRustGo modules.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub code: String,
    pub module: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestContext {
    pub module: String,
    pub public_api: Vec<String>,
    pub historical_bugs: Vec<String>,
    pub edge_cases: Vec<String>,
}

#[derive(Debug, Error)]
pub enum TestGenError {
    #[error("API call failed: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub struct AITestGenerator {
    client: reqwest::Client,
    api_endpoint: String,
    model: String,
}

impl AITestGenerator {
    pub fn new(api_endpoint: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_endpoint,
            model,
        }
    }

    pub async fn generate_tests(
        &self,
        module: &str,
        context: &TestContext,
    ) -> Result<Vec<TestCase>, TestGenError> {
        let prompt = format!(
            r#"
Generate comprehensive test cases for SQLRustGo {} module.

Context:
- Module: {}
- Public API: {:?}
- Historical bugs: {:?}
- Edge cases: {:?}

Generate test cases covering:
1. Happy path scenarios
2. Edge cases and boundary conditions
3. Error handling
4. Performance regression tests
5. Concurrency tests

Output format: JSON array of test cases with fields: name, code, module, description
"#,
            module,
            module,
            context.public_api,
            context.historical_bugs,
            context.edge_cases
        );

        let response = self.call_llm(&prompt).await?;
        self.parse_test_cases(&response, module)
    }

    async fn call_llm(&self, prompt: &str) -> Result<String, TestGenError> {
        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        });

        let response = self.client
            .post(&self.api_endpoint)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TestGenError::ApiError(e.to_string()))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TestGenError::ApiError(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| TestGenError::ApiError("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    fn parse_test_cases(&self, response: &str, module: &str) -> Result<Vec<TestCase>, TestGenError> {
        // Try to extract JSON array from response
        let json_str = response
            .lines()
            .skip_while(|l| !l.contains('['))
            .take_while(|l| !l.contains("```") || l.contains('['))
            .collect::<Vec<_>>()
            .join("\n");

        serde_json::from_str(&json_str)
            .map_err(|e| TestGenError::ParseError(format!("Failed to parse JSON: {}. Response: {}", e, json_str)))
    }
}
