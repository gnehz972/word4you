use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: String,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model_name: String) -> Self {
        let base_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model_name
        );
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    pub async fn get_word_explanation(&self, word: &str, prompt_template: &str) -> Result<String> {
        let prompt = prompt_template.replace("[INSERT WORD HERE]", &word.to_lowercase());

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let url = format!("{}?key={}", self.base_url, self.api_key);

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone().trim().to_string());
            }
        }

        Err(anyhow!("No response received from Gemini API"))
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: "Hello".to_string(),
                }],
            }],
        };

        let url = format!("{}?key={}", self.base_url, self.api_key);

        let response = self.client.post(&url).json(&request).send().await;

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
