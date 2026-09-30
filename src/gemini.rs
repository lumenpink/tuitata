use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub filename: String,
    pub mime_type: String,
    pub data_base64: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" ou "model"
    pub content: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<ImageAttachment>,
}

#[derive(Serialize)]
struct InlineData<'a> {
    #[serde(rename = "mimeType")]
    mime_type: &'a str,
    data: &'a str,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GeminiPart<'a> {
    Text {
        text: &'a str,
    },
    Inline {
        #[serde(rename = "inlineData")]
        inline_data: InlineData<'a>,
    },
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
            .map(|m| {
                let mut parts = vec![GeminiPart::Text { text: &m.content }];
                if let Some(att) = &m.attachment {
                    parts.push(GeminiPart::Inline {
                        inline_data: InlineData {
                            mime_type: &att.mime_type,
                            data: &att.data_base64,
                        },
                    });
                }
                GeminiContent {
                    role: if m.role == "user" { "user" } else { "model" },
                    parts,
                }
            })
            .collect();

        let sys_prompt = "Você é o Tuitatá, um assistente técnico de alta precisão integrado ao ambiente de terminal Linux. Quando sugerir código ou criar/modificar arquivos, especifique o caminho do arquivo no identificador do bloco markdown como ```file:caminho/do/arquivo.ext ou ```linguagem:caminho/do/arquivo.ext, permitindo que o usuário grave diretamente no disco.";
        let payload = GeminiRequest {
            system_instruction: Some(GeminiSystemInstruction {
                parts: vec![GeminiPart::Text { text: sys_prompt }],
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

    pub async fn generate_image(&self, prompt: &str) -> Result<(String, Vec<u8>), String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-image:generateContent?key={}",
            self.api_key
        );

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Falha ao conectar com endpoint de imagem: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Erro ao gerar imagem ({status}): {err_text}"));
        }

        let body: Value = resp.json().await.map_err(|e| format!("Erro ao processar JSON da imagem: {}", e))?;

        let parts = body
            .pointer("/candidates/0/content/parts")
            .and_then(|p| p.as_array())
            .ok_or_else(|| "Resposta não contém blocos válidos de dados".to_string())?;

        for part in parts {
            if let Some(inline) = part.get("inlineData") {
                let mime = inline
                    .get("mimeType")
                    .and_then(|m| m.as_str())
                    .unwrap_or("image/jpeg")
                    .to_string();
                let b64 = inline
                    .get("data")
                    .and_then(|d| d.as_str())
                    .ok_or_else(|| "Bloco de dados sem base64".to_string())?;

                use base64::Engine;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| format!("Erro ao decodificar base64 da imagem: {}", e))?;

                return Ok((mime, bytes));
            }
        }

        Err("Nenhuma imagem retornada pelo modelo".to_string())
    }
}
