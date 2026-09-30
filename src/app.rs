use crate::config::Config;
use crate::gemini::ChatMessage;
use chrono::Local;
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;

pub struct App {
    pub config: Config,
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub cursor_pos: usize,
    pub is_generating: bool,
    pub current_model: String,
    pub scroll: usize,
    pub max_scroll: Cell<usize>,
    pub status: Option<(String, std::time::Instant)>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let current_model = config.flash_model.clone();
        Self {
            config,
            messages: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            is_generating: false,
            current_model,
            scroll: 0,
            max_scroll: Cell::new(0),
            status: None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let mut chars: Vec<char> = self.input.chars().collect();
        if self.cursor_pos > chars.len() {
            self.cursor_pos = chars.len();
        }
        chars.insert(self.cursor_pos, c);
        self.input = chars.into_iter().collect();
        self.cursor_pos += 1;
    }

    pub fn delete_char_before(&mut self) {
        if self.cursor_pos > 0 {
            let mut chars: Vec<char> = self.input.chars().collect();
            if self.cursor_pos <= chars.len() {
                chars.remove(self.cursor_pos - 1);
                self.input = chars.into_iter().collect();
                self.cursor_pos -= 1;
            }
        }
    }

    pub fn delete_char_after(&mut self) {
        let mut chars: Vec<char> = self.input.chars().collect();
        if self.cursor_pos < chars.len() {
            chars.remove(self.cursor_pos);
            self.input = chars.into_iter().collect();
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        let len = self.input.chars().count();
        if self.cursor_pos < len {
            self.cursor_pos += 1;
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_pos = 0;
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status = Some((msg.to_string(), std::time::Instant::now()));
    }

    pub fn toggle_model(&mut self) {
        if self.current_model == self.config.flash_model {
            self.current_model = self.config.pro_model.clone();
            self.set_status("Alternado para modelo: 🧠 Gemini Pro (Raciocínio Profundo)");
        } else if self.current_model == self.config.pro_model {
            self.current_model = self.config.ollama_model.clone();
            self.set_status(&format!("Alternado para modelo: 🦙 Ollama Local ({})", self.config.ollama_model));
        } else {
            self.current_model = self.config.flash_model.clone();
            self.set_status("Alternado para modelo: ⚡ Gemini Flash (Ultrarrápido)");
        }
    }

    pub fn set_model_by_name(&mut self, target: &str) {
        let t = target.to_lowercase();
        if t.contains("ollama") || t.contains("local") || t.contains("qwen") {
            self.current_model = self.config.ollama_model.clone();
            self.set_status(&format!("Alternado para modelo: 🦙 Ollama Local ({})", self.config.ollama_model));
        } else if t.contains("pro") {
            self.current_model = self.config.pro_model.clone();
            self.set_status("Alternado para modelo: 🧠 Gemini Pro (Raciocínio Profundo)");
        } else {
            self.current_model = self.config.flash_model.clone();
            self.set_status("Alternado para modelo: ⚡ Gemini Flash (Ultrarrápido)");
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        let now = Local::now().format("%H:%M").to_string();
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content,
            timestamp: now,
        });
        self.scroll_to_bottom();
    }

    pub fn append_to_last_model_message(&mut self, text: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == "model" {
                last.content.push_str(text);
                self.scroll_to_bottom();
                return;
            }
        }
        let now = Local::now().format("%H:%M").to_string();
        self.messages.push(ChatMessage {
            role: "model".to_string(),
            content: text.to_string(),
            timestamp: now,
        });
        self.scroll_to_bottom();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = usize::MAX;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_up_lines(&mut self, n: usize) {
        let max = self.max_scroll.get();
        let current = if self.scroll > max {
            max
        } else {
            self.scroll
        };
        self.scroll = current.saturating_sub(n);
    }

    pub fn scroll_down_lines(&mut self, n: usize) {
        let max = self.max_scroll.get();
        let current = if self.scroll > max {
            max
        } else {
            self.scroll
        };
        let next = current.saturating_add(n);
        if next >= max {
            self.scroll = usize::MAX;
        } else {
            self.scroll = next;
        }
    }

    pub fn copy_last_code_block(&mut self) -> Result<(), String> {
        let code = self.extract_last_code_block().ok_or("Nenhum bloco de código encontrado nas respostas!")?;
        let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("Clipboard erro: {}", e))?;
        clipboard.set_text(code).map_err(|e| format!("Clipboard erro: {}", e))?;
        self.set_status("📋 Bloco de código copiado para a área de transferência!");
        Ok(())
    }

    pub fn extract_last_file_or_code_block(&self) -> Option<(Option<String>, String)> {
        for msg in self.messages.iter().rev() {
            if msg.role == "model" {
                if let Some(start) = msg.content.rfind("```") {
                    let rest = &msg.content[..start];
                    if let Some(prev) = rest.rfind("```") {
                        let header_and_body = &msg.content[prev + 3..start];
                        let (header, body) = if let Some(newline) = header_and_body.find('\n') {
                            (&header_and_body[..newline], &header_and_body[newline + 1..])
                        } else {
                            ("", header_and_body)
                        };

                        let header_trimmed = header.trim();
                        let mut suggested_path = None;

                        // Verifica header no formato `file:<caminho>`, `path:<caminho>` ou `lang:<caminho>`
                        if let Some(pos) = header_trimmed.find(':') {
                            let possible_path = header_trimmed[pos + 1..].trim();
                            if !possible_path.is_empty() {
                                suggested_path = Some(possible_path.to_string());
                            }
                        }

                        // Se não encontrou no header, verifica na primeira linha de comentário do código
                        if suggested_path.is_none() {
                            if let Some(first_line) = body.lines().next() {
                                let trimmed_line = first_line.trim();
                                if let Some(path_candidate) = trimmed_line.strip_prefix("//")
                                    .or_else(|| trimmed_line.strip_prefix('#'))
                                    .or_else(|| trimmed_line.strip_prefix("/*"))
                                {
                                    let clean = path_candidate.trim().trim_end_matches("*/").trim();
                                    if (clean.contains('/') || clean.contains('.')) && !clean.contains(' ') {
                                        suggested_path = Some(clean.to_string());
                                    }
                                }
                            }
                        }

                        return Some((suggested_path, body.trim().to_string()));
                    }
                }
            }
        }
        None
    }

    fn extract_last_code_block(&self) -> Option<String> {
        self.extract_last_file_or_code_block().map(|(_, code)| code)
    }

    pub fn save_last_code_to_file(&mut self, explicit_path: Option<&str>) -> Result<PathBuf, String> {
        let (suggested, code) = self.extract_last_file_or_code_block()
            .ok_or_else(|| {
                let err = "Nenhum bloco de código encontrado para salvar!".to_string();
                self.set_status(&format!("⚠️ {}", err));
                err
            })?;

        let target_str = match explicit_path {
            Some(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => match suggested {
                Some(p) => p,
                None => {
                    let err = "Nenhum caminho detectado. Use /save <caminho> para salvar.".to_string();
                    self.set_status(&format!("⚠️ {}", err));
                    return Err(err);
                }
            },
        };

        let target_path = crate::context::expand_path(&target_str);
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(&target_path, code.as_bytes())
            .map_err(|e| {
                let err = format!("Falha ao gravar {}: {}", target_path.display(), e);
                self.set_status(&format!("⚠️ {}", err));
                err
            })?;

        self.set_status(&format!("💾 Arquivo gravado: {} ({} bytes)", target_path.display(), code.len()));
        Ok(target_path)
    }

    pub fn export_to_obsidian(&mut self) -> Result<PathBuf, String> {
        if self.messages.is_empty() {
            return Err("Nenhuma mensagem para exportar!".to_string());
        }

        let now = Local::now();
        let timestamp_str = now.format("%Y-%m-%d %H:%M").to_string();
        let file_name = format!("{}-tuitata.md", now.format("%Y%m%d_%H%M%S"));

        let inbox = self.config.obsidian_vault.join("01_inbox");
        let _ = fs::create_dir_all(&inbox);
        let target_path = inbox.join(&file_name);

        let mut doc = format!(
            "---\ndata: {}\nmodelo: {}\ntags:\n  - tuitata\n  - gemini\n  - chat\n---\n\n# Conversa Tuitatá — {}\n\n",
            timestamp_str, self.current_model, timestamp_str
        );

        for m in &self.messages {
            let role_label = if m.role == "user" { "👤 **Você**" } else { "🔥 **Tuitatá**" };
            doc.push_str(&format!("### {} ({})\n\n{}\n\n---\n\n", role_label, m.timestamp, m.content));
        }

        fs::write(&target_path, doc).map_err(|e| format!("Falha ao salvar no Obsidian: {}", e))?;
        self.set_status(&format!("📝 Conversa salva no Obsidian: 01_inbox/{}", file_name));
        Ok(target_path)
    }
}
