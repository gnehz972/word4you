use crate::ai_client::AiClient;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct QwenRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct QwenResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
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
        let base_url =
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string();
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }
}

#[async_trait::async_trait]
impl AiClient for QwenClient {
    async fn get_text_explanation(&self, text: &str, prompt_template: &str) -> Result<String> {
        let prompt = prompt_template.replace("[INSERT TEXT HERE]", text);

        let request = QwenRequest {
            model: "qwen-turbo".to_string(), // Default model, can be overridden
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.7,
            max_tokens: 1000,
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

        if let Some(choice) = qwen_response.choices.first() {
            return Ok(choice.message.content.clone().trim().to_string());
        }

        Err(anyhow!("No response received from QWEN API"))
    }

    async fn test_connection(&self) -> Result<bool> {
        let request = QwenRequest {
            model: "qwen-turbo".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: 0.7,
            max_tokens: 10,
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
        let client = QwenClient::new("test_api_key".to_string(), "qwen-turbo".to_string());

        assert_eq!(client.api_key, "test_api_key");
        assert_eq!(
            client.base_url,
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
    }
}
