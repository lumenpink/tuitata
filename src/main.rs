mod app;
mod config;
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

    // Se argumentos foram passados diretamente no terminal (modo CLI pontual: `tt "dúvida"`)
    if !args.is_empty() {
        let prompt = args.join(" ");
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

                    // F2: Alternar entre Flash e Pro
                    if key.code == KeyCode::F(2) {
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

                    // Scroll no chat
                    if key.code == KeyCode::PageUp {
                        app.scroll_up();
                        continue;
                    }
                    if key.code == KeyCode::PageDown {
                        app.scroll_down();
                        continue;
                    }

                    // Envio de mensagem com Enter
                    if key.code == KeyCode::Enter && !app.is_generating {
                        let text = app.input.trim().to_string();
                        if !text.is_empty() {
                            app.add_user_message(text);
                            app.input.clear();
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

                    // Digitação na barra de entrada
                    match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
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
