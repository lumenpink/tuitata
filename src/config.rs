use std::fs;
use std::path::PathBuf;

pub const DEFAULT_FLASH_MODEL: &str = "gemini-3.8-flash";
pub const DEFAULT_PRO_MODEL: &str = "gemini-3.1-pro-preview";
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5-coder:3b";
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

#[derive(Clone, Debug)]
pub struct Config {
    pub api_key: String,
    pub flash_model: String,
    pub pro_model: String,
    pub ollama_model: String,
    pub ollama_url: String,
    pub obsidian_vault: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let api_key = Self::find_api_key().ok_or_else(|| {
            "GEMINI_API_KEY não encontrada! Configure no ~/.env ou como variável de ambiente.".to_string()
        })?;

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/lumen"));
        let obsidian_vault = home.join("obsidian");

        let ollama_model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.to_string());
        let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());

        Ok(Self {
            api_key,
            flash_model: DEFAULT_FLASH_MODEL.to_string(),
            pro_model: DEFAULT_PRO_MODEL.to_string(),
            ollama_model,
            ollama_url,
            obsidian_vault,
        })
    }

    fn find_api_key() -> Option<String> {
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            let trimmed = key.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }

        // Tentar ler de ~/.env
        if let Some(home) = dirs::home_dir() {
            let env_path = home.join(".env");
            if let Ok(content) = fs::read_to_string(&env_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("GEMINI_API_KEY=") {
                        let val = trimmed.split_once('=').map(|(_, v)| v.trim().trim_matches('"').trim_matches('\''));
                        if let Some(k) = val {
                            if !k.is_empty() {
                                return Some(k.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
