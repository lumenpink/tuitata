use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" ou "model"
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    role: &'a str,
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct GeminiSystemInstruction<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct GeminiRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction<'a>>,
    contents: Vec<GeminiContent<'a>>,
}

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { client, api_key }
    }

    pub async fn stream_chat(
        &self,
        messages: &[ChatMessage],
        model: &str,
        tx: UnboundedSender<String>,
    ) -> Result<(), String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            model, self.api_key
        );

        let contents: Vec<GeminiContent> = messages
            .iter()
            .map(|m| GeminiContent {
                role: if m.role == "user" { "user" } else { "model" },
                parts: vec![GeminiPart { text: &m.content }],
            })
            .collect();

        let sys_prompt = "Você é o Tuitatá, um assistente técnico de alta precisão integrado ao ambiente de terminal Linux. Quando sugerir código ou criar/modificar arquivos, especifique o caminho do arquivo no identificador do bloco markdown como ```file:caminho/do/arquivo.ext ou ```linguagem:caminho/do/arquivo.ext, permitindo que o usuário grave diretamente no disco.";
        let payload = GeminiRequest {
            system_instruction: Some(GeminiSystemInstruction {
                parts: vec![GeminiPart { text: sys_prompt }],
            }),
            contents,
        };

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com Gemini: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Erro da API ({status}): {err_text}"));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| format!("Erro no stream: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..=pos);

                if let Some(data) = line.strip_prefix("data:") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(val) = serde_json::from_str::<Value>(data) {
                        if let Some(text) = val
                            .pointer("/candidates/0/content/parts/0/text")
                            .and_then(|t| t.as_str())
                        {
                            let _ = tx.send(text.to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
