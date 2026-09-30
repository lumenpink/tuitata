mod app;
mod config;
mod context;
mod gemini;
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

    let client = GeminiClient::new(config.api_key.clone());

    // Se argumentos foram passados diretamente no terminal (modo CLI pontual: `tt "dúvida"` ou `tt @arquivo "pergunta"`)
    if !args.is_empty() {
        let raw_prompt = args.join(" ");
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
        }];

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let model = config.flash_model.clone();

        tokio::spawn(async move {
            if let Err(e) = client.stream_chat(&messages, &model, tx).await {
                eprintln!("\n[Tuitatá Erro] {}", e);
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

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            // Recebimento de chunks de streaming da API do Gemini
            Some(chunk) = chunk_rx.recv() => {
                app.append_to_last_model_message(&chunk);
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
                            if text == "/model" || text == "/m" {
                                app.toggle_model();
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

                            app.add_user_message(prompt_text);
                            app.clear_input();
                            app.is_generating = true;

                            let client_clone = GeminiClient::new(app.config.api_key.clone());
                            let messages_snapshot = app.messages.clone();
                            let model_name = app.current_model.clone();
                            let tx = chunk_tx.clone();

                            tokio::spawn(async move {
                                let _ = client_clone.stream_chat(&messages_snapshot, &model_name, tx).await;
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

        // Se terminou a geração de streaming
        if app.is_generating && chunk_rx.is_empty() {
            // Pequeno delay para liberar a trava do gerador
            app.is_generating = false;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
