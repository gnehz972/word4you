use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::ai_client::AiClient;

#[derive(Debug, Serialize)]
struct QwenRequest {
    model: String,
    input: QwenInput,
    parameters: QwenParameters,
}

#[derive(Debug, Serialize)]
struct QwenInput {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct QwenParameters {
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct QwenResponse {
    output: QwenOutput,
    usage: Option<QwenUsage>,
}

#[derive(Debug, Deserialize)]
struct QwenOutput {
    text: String,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QwenUsage {
    total_tokens: Option<u32>,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

pub struct QwenClient {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
}

impl QwenClient {
    pub fn new(api_key: String, _model_name: String) -> Self {
        let base_url = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string();
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }
}

#[async_trait::async_trait]
impl AiClient for QwenClient {
    async fn get_word_explanation(&self, word: &str, prompt_template: &str) -> Result<String> {
        let prompt = prompt_template.replace("[INSERT WORD HERE]", &word.to_lowercase());

        let request = QwenRequest {
            model: "qwen-turbo".to_string(), // Default model, can be overridden
            input: QwenInput {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: prompt,
                }],
            },
            parameters: QwenParameters {
                temperature: 0.7,
                max_tokens: 1000,
            },
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("QWEN API error: {}", error_text));
        }

        let qwen_response: QwenResponse = response.json().await?;

        Ok(qwen_response.output.text.trim().to_string())
    }

    async fn test_connection(&self) -> Result<bool> {
        let request = QwenRequest {
            model: "qwen-turbo".to_string(),
            input: QwenInput {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                }],
            },
            parameters: QwenParameters {
                temperature: 0.7,
                max_tokens: 10,
            },
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qwen_client_creation() {
        let client = QwenClient::new(
            "test_api_key".to_string(),
            "qwen-turbo".to_string(),
        );
        
        assert_eq!(client.api_key, "test_api_key");
        assert_eq!(client.base_url, "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation");
    }
} 