mod app;
mod config;
mod context;
mod gemini;
mod media;
mod ollama;
mod ui;

use app::App;
use config::Config;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use gemini::GeminiClient;
use ollama::OllamaClient;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Tuitatá Erro] {}", e);
            std::process::exit(1);
        }
    };

    // Se argumentos foram passados diretamente no terminal (modo CLI pontual: `tt "dúvida"` ou `tt --ollama "dúvida"`)
    if !args.is_empty() {
        let (use_ollama, clean_args): (bool, Vec<String>) = if args.iter().any(|a| a == "--ollama" || a == "-o") {
            (true, args.into_iter().filter(|a| a != "--ollama" && a != "-o").collect())
        } else {
            (false, args)
        };

        let raw_prompt = clean_args.join(" ");
        let (prompt, _) = match context::process_input_files(&raw_prompt) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("[Tuitatá Erro de Arquivo] {}", e);
                std::process::exit(1);
            }
        };

        let messages = vec![gemini::ChatMessage {
            role: "user".to_string(),
            content: prompt,
            timestamp: String::new(),
            attachment: None,
        }];

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let client_gemini = GeminiClient::new(config.api_key.clone());
        let client_ollama = OllamaClient::new(Some(config.ollama_url.clone()));
        let model = if use_ollama {
            config.ollama_model.clone()
        } else {
            config.flash_model.clone()
        };

        tokio::spawn(async move {
            if use_ollama {
                if let Err(e) = client_ollama.stream_chat(&messages, &model, tx).await {
                    eprintln!("\n[Tuitatá Erro Ollama] {}", e);
                }
            } else {
                if let Err(e) = client_gemini.stream_chat(&messages, &model, tx).await {
                    eprintln!("\n[Tuitatá Erro Gemini] {}", e);
                }
            }
        });

        while let Some(chunk) = rx.recv().await {
            print!("{}", chunk);
            io::Write::flush(&mut stdout())?;
        }
        println!();
        return Ok(());
    }

    // Modo TUI Interativo
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);
    let mut event_stream = EventStream::new();
    let (chunk_tx, mut chunk_rx) = mpsc::unbounded_channel::<String>();
    let (done_tx, mut done_rx) = mpsc::unbounded_channel::<()>();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            // Recebimento de chunks de streaming da API do Gemini ou Ollama
            Some(chunk) = chunk_rx.recv() => {
                app.append_to_last_model_message(&chunk);
            }

            // Conclusão da tarefa assíncrona de geração
            Some(()) = done_rx.recv() => {
                app.is_generating = false;
            }

            // Leitura de eventos de teclado do terminal
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    // Sair do app com ESC ou Ctrl+C
                    if key.code == KeyCode::Esc || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c')) {
                        break;
                    }

                    // Alternar modelo: Ctrl+P, Tab (quando input vazio) ou F2 (legado)
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('p'))
                        || (key.code == KeyCode::Tab && app.input.is_empty())
                        || key.code == KeyCode::F(2)
                    {
                        app.toggle_model();
                        continue;
                    }

                    // Ctrl+V ou Alt+V: Colar imagem diretamente do Clipboard do sistema
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('v'))
                        || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('v'))
                    {
                        if let Err(e) = app.attach_clipboard_image() {
                            app.set_status(&format!("⚠️ {}", e));
                        }
                        continue;
                    }

                    // Ctrl+I: Atalho para anexar imagem via caminho
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('i') {
                        app.input = "/image ".to_string();
                        app.cursor_pos = app.input.chars().count();
                        continue;
                    }

                    // Ctrl+X: Limpar anexo de imagem pendente se houver
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('x') {
                        if app.pending_image.is_some() {
                            app.clear_pending_image();
                            continue;
                        }
                    }

                    // Ctrl+S: Exportar para o Obsidian
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('s') {
                        let _ = app.export_to_obsidian();
                        continue;
                    }

                    // Alt+Y: Copiar último bloco de código para o Clipboard
                    if (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('y'))
                        || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('y')) {
                        let _ = app.copy_last_code_block();
                        continue;
                    }

                    // Ctrl+W: Gravar arquivo proposto ou último bloco no disco
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('w') {
                        let _ = app.save_last_code_to_file(None);
                        continue;
                    }

                    // Ctrl+F: Próxima ocorrência de busca ou iniciar busca no prompt
                    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('f') {
                        if app.search_query.is_some() && !app.search_matches.is_empty() {
                            app.next_search_match();
                        } else {
                            app.input = "/find ".to_string();
                            app.cursor_pos = app.input.chars().count();
                        }
                        continue;
                    }

                    // Scroll no chat (otimizado para tablet e terminal padrão)
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('u'))
                        || key.code == KeyCode::PageUp
                    {
                        app.scroll_up_lines(10);
                        continue;
                    }
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('d'))
                        || key.code == KeyCode::PageDown
                    {
                        app.scroll_down_lines(10);
                        continue;
                    }
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('k'))
                        || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Up)
                    {
                        app.scroll_up_lines(1);
                        continue;
                    }
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('j'))
                        || (key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Down)
                    {
                        app.scroll_down_lines(1);
                        continue;
                    }

                    // Envio de mensagem com Enter
                    if key.code == KeyCode::Enter && !app.is_generating {
                        let text = app.input.trim().to_string();
                        if !text.is_empty() {
                            if text == "/clear_find" || text == "/nofind" {
                                app.clear_search();
                                app.set_status("Busca textual limpa.");
                                app.clear_input();
                                continue;
                            }
                            if text.starts_with("/find ") || text.starts_with("/busca ") {
                                let query = if text.starts_with("/find ") {
                                    text.trim_start_matches("/find ").trim()
                                } else {
                                    text.trim_start_matches("/busca ").trim()
                                };
                                app.search_in_messages(query);
                                app.clear_input();
                                continue;
                            }
                            if text.starts_with('/') && !text.contains(' ') && text.len() > 1 {
                                let word = text.trim_start_matches('/');
                                let known_cmds = [
                                    "model", "m", "top", "bottom", "inicio", "fim", "write", "apply", "gravar",
                                    "save", "salvar", "file", "ler", "clear_find", "nofind", "image", "imagem",
                                    "noimage", "limpar_imagem", "imagine", "gerar"
                                ];
                                if !known_cmds.contains(&word) {
                                    app.search_in_messages(word);
                                    app.clear_input();
                                    continue;
                                }
                            }
                            if text == "/model" || text == "/m" {
                                app.toggle_model();
                                app.clear_input();
                                continue;
                            }
                            if text.starts_with("/model ") || text.starts_with("/m ") {
                                let name = if text.starts_with("/model ") {
                                    text.trim_start_matches("/model ").trim()
                                } else {
                                    text.trim_start_matches("/m ").trim()
                                };
                                app.set_model_by_name(name);
                                app.clear_input();
                                continue;
                            }
                            if text == "/top" || text == "/inicio" {
                                app.scroll_to_top();
                                app.clear_input();
                                continue;
                            }
                            if text == "/bottom" || text == "/fim" {
                                app.scroll_to_bottom();
                                app.clear_input();
                                continue;
                            }
                            if text == "/write" || text == "/apply" || text == "/gravar" {
                                let _ = app.save_last_code_to_file(None);
                                app.clear_input();
                                continue;
                            }
                            if text.starts_with("/save ") || text.starts_with("/salvar ") {
                                let path = if text.starts_with("/save ") {
                                    text.trim_start_matches("/save ").trim()
                                } else {
                                    text.trim_start_matches("/salvar ").trim()
                                };
                                let _ = app.save_last_code_to_file(Some(path));
                                app.clear_input();
                                continue;
                            }

                            // Comandos de Imagem (Visão e Geração)
                            if text.starts_with("/image ") || text.starts_with("/imagem ") {
                                let path = if text.starts_with("/image ") {
                                    text.trim_start_matches("/image ").trim()
                                } else {
                                    text.trim_start_matches("/imagem ").trim()
                                };
                                if let Err(e) = app.attach_image_file(path) {
                                    app.set_status(&format!("⚠️ {}", e));
                                }
                                app.clear_input();
                                continue;
                            }

                            if text == "/noimage" || text == "/limpar_imagem" {
                                app.clear_pending_image();
                                app.clear_input();
                                continue;
                            }

                            // Geração de Imagem com Gemini Flash Image
                            if text.starts_with("/imagine ") || text.starts_with("/gerar ") {
                                let prompt = if text.starts_with("/imagine ") {
                                    text.trim_start_matches("/imagine ").trim()
                                } else {
                                    text.trim_start_matches("/gerar ").trim()
                                }.to_string();

                                if prompt.is_empty() {
                                    app.set_status("⚠️ Especifique a descrição da imagem (ex: /imagine capivara de oculos)");
                                    app.clear_input();
                                    continue;
                                }

                                app.add_user_message(format!("🎨 /imagine {}", prompt), None);
                                app.clear_input();
                                app.is_generating = true;
                                app.set_status("🎨 Gerando imagem com Gemini Flash Image...");

                                let client_gemini = GeminiClient::new(app.config.api_key.clone());
                                let vault_dir = app.config.obsidian_vault.clone();
                                let p_clone = prompt.clone();
                                let tx = chunk_tx.clone();
                                let d_tx = done_tx.clone();

                                tokio::spawn(async move {
                                    match client_gemini.generate_image(&p_clone).await {
                                        Ok((mime, bytes)) => {
                                            match media::save_generated_image(&vault_dir, &p_clone, &mime, &bytes) {
                                                Ok(saved_path) => {
                                                    media::open_image_viewer(&saved_path);
                                                    let resp = format!(
                                                        "✨ **Imagem gerada com sucesso!**\n\n📁 **Salva em:** `{}`\n🖼️ Visualizador `nsxiv` disparado.",
                                                        saved_path.display()
                                                    );
                                                    let _ = tx.send(resp);
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(format!("⚠️ Erro ao salvar imagem no Obsidian: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = tx.send(format!("❌ Falha ao gerar imagem: {}", e));
                                        }
                                    }
                                    let _ = d_tx.send(());
                                });
                                continue;
                            }

                            let (prompt_text, attached_files) = match context::process_input_files(&text) {
                                Ok(res) => res,
                                Err(err_msg) => {
                                    app.set_status(&format!("⚠️ {}", err_msg));
                                    continue;
                                }
                            };

                            if !attached_files.is_empty() {
                                app.set_status(&format!("📎 Anexado: {}", attached_files.join(", ")));
                            }

                            let attachment = app.pending_image.take();
                            app.add_user_message(prompt_text, attachment);
                            app.clear_input();
                            app.is_generating = true;

                            let client_gemini = GeminiClient::new(app.config.api_key.clone());
                            let client_ollama = OllamaClient::new(Some(app.config.ollama_url.clone()));
                            let messages_snapshot = app.messages.clone();
                            let model_name = app.current_model.clone();
                            let is_ollama = model_name == app.config.ollama_model
                                || model_name.contains("qwen")
                                || model_name.contains("ollama");
                            let tx = chunk_tx.clone();
                            let d_tx = done_tx.clone();

                            tokio::spawn(async move {
                                if is_ollama {
                                    if let Err(e) = client_ollama.stream_chat(&messages_snapshot, &model_name, tx.clone()).await {
                                        let _ = tx.send(format!("\n[Tuitatá Erro Ollama] {}", e));
                                    }
                                } else {
                                    if let Err(e) = client_gemini.stream_chat(&messages_snapshot, &model_name, tx.clone()).await {
                                        let _ = tx.send(format!("\n[Tuitatá Erro Gemini] {}", e));
                                    }
                                }
                                let _ = d_tx.send(());
                            });
                        }
                        continue;
                    }

                    // Digitação e navegação na barra de entrada
                    match key.code {
                        KeyCode::Char(c) => {
                            app.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            app.delete_char_before();
                        }
                        KeyCode::Delete => {
                            app.delete_char_after();
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
