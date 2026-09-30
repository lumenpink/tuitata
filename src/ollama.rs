use crate::gemini::ChatMessage;
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    images: Vec<&'a str>,
}

#[derive(Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage<'a>>,
    stream: bool,
}

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();
        Self { client, base_url }
    }

    pub async fn stream_chat(
        &self,
        messages: &[ChatMessage],
        model: &str,
        tx: UnboundedSender<String>,
    ) -> Result<(), String> {
        let url = format!("{}/api/chat", self.base_url);

        let sys_prompt = "Você é o Tuitatá operando com modelo local Ollama. Quando criar ou alterar arquivos, especifique o caminho no bloco markdown com ```file:caminho/do/arquivo ou ```linguagem:caminho/do/arquivo para que o usuário possa gravar diretamente.";

        let mut req_messages = Vec::new();
        req_messages.push(OllamaMessage {
            role: "system",
            content: sys_prompt,
            images: Vec::new(),
        });

        for m in messages {
            let images = if let Some(att) = &m.attachment {
                vec![att.data_base64.as_str()]
            } else {
                Vec::new()
            };

            req_messages.push(OllamaMessage {
                role: if m.role == "user" { "user" } else { "assistant" },
                content: &m.content,
                images,
            });
        }

        let payload = OllamaChatRequest {
            model,
            messages: req_messages,
            stream: true,
        };

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Falha ao conectar com o Ollama em {}: {}. O serviço está ativo?", self.base_url, e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Erro da API Ollama ({status}): {err_text}"));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| format!("Erro no stream do Ollama: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..=pos);

                if line.is_empty() {
                    continue;
                }

                if let Ok(val) = serde_json::from_str::<Value>(&line) {
                    if let Some(content) = val
                        .pointer("/message/content")
                        .and_then(|c| c.as_str())
                    {
                        let _ = tx.send(content.to_string());
                    }
                    if val.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
