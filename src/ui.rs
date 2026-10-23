use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Chat Messages
            Constraint::Length(3), // Input Area
            Constraint::Length(1), // Footer / Status Bar
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_chat(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
    render_footer(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let model_badge = if app.current_model == app.config.ollama_model
        || app.current_model.contains("qwen")
        || app.current_model.contains("ollama")
    {
        Span::styled(
            format!(" [🦙 Ollama: {}] ", app.config.ollama_model),
            Style::default()
                .fg(Color::Rgb(100, 255, 100))
                .add_modifier(Modifier::BOLD),
        )
    } else if app.current_model.contains("pro") {
        Span::styled(
            " [🧠 Gemini Pro] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [⚡ Gemini Flash] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    };

    let title = Line::from(vec![
        Span::styled(" 🔥 TUITATÁ ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
        Span::styled("(`tt`)", Style::default().fg(Color::DarkGray)),
        model_badge,
        Span::styled(" │ <Ctrl+P/Tab> Modelo │ <Ctrl+V> Colar Imagem │ /imagine <prompt> │ <Ctrl+S> Salvar │ <Esc> Sair", Style::default().fg(Color::Gray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 60, 140))); // Electric Pink

    let p = Paragraph::new(title).block(block);
    f.render_widget(p, area);
}

fn highlight_line<'a>(
    prefix: &'a str,
    content: &'a str,
    base_style: Style,
    query: Option<&str>,
) -> Line<'a> {
    let q = match query {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Line::from(vec![
                Span::styled(prefix, base_style),
                Span::styled(content, base_style),
            ])
        }
    };

    let q_lower = q.to_lowercase();
    let content_lower = content.to_lowercase();

    if !content_lower.contains(&q_lower) {
        return Line::from(vec![
            Span::styled(prefix, base_style),
            Span::styled(content, base_style),
        ]);
    }

    let mut spans = Vec::new();
    if !prefix.is_empty() {
        spans.push(Span::styled(prefix, base_style));
    }

    let mut current_idx = 0;
    while let Some(rel_pos) = content_lower[current_idx..].find(&q_lower) {
        let match_start = current_idx + rel_pos;
        let match_end = match_start + q.len();

        if match_start > current_idx {
            spans.push(Span::styled(&content[current_idx..match_start], base_style));
        }

        spans.push(Span::styled(
            &content[match_start..match_end],
            Style::default()
                .bg(Color::Rgb(255, 120, 0))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        current_idx = match_end;
    }

    if current_idx < content.len() {
        spans.push(Span::styled(&content[current_idx..], base_style));
    }

    Line::from(spans)
}

fn render_chat(f: &mut Frame, app: &App, area: Rect) {
    let title = if let Some(ref q) = app.search_query {
        format!(" Conversa [🔍 Busca: '{}'] ", q)
    } else {
        " Conversa ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title);

    let inner_area = block.inner(area);
    let mut lines = Vec::new();

    if app.messages.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "✨ Tuitatá pronto. Digite sua pergunta abaixo e aperte Enter!",
            Style::default().fg(Color::DarkGray),
        )]));
    }

    for msg in &app.messages {
        let is_user = msg.role == "user";
        let (prefix, color) = if is_user {
            (
                format!("👤 Você ({})", msg.timestamp),
                Color::Rgb(0, 230, 255),
            ) // Neon Cyan
        } else {
            (
                format!("🔥 Tuitatá ({})", msg.timestamp),
                Color::Rgb(255, 60, 140),
            ) // Electric Pink
        };

        lines.push(Line::from(vec![Span::styled(
            prefix,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )]));

        if let Some(att) = &msg.attachment {
            lines.push(Line::from(vec![
                Span::styled(
                    "  📎 [Imagem: ",
                    Style::default().fg(Color::Rgb(150, 255, 150)),
                ),
                Span::styled(
                    &att.filename,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ({})]", att.mime_type),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        let mut in_code_block = false;
        let query = app.search_query.as_deref();
        for line_str in msg.content.lines() {
            if line_str.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", line_str),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::DIM),
                )]));
                continue;
            }

            if in_code_block {
                let base_style = Style::default().fg(Color::Rgb(255, 230, 100)); // Acid Yellow
                lines.push(highlight_line("    ", line_str, base_style, query));
            } else {
                let base_style = Style::default().fg(Color::White);
                lines.push(highlight_line("  ", line_str, base_style, query));
            }
        }
        lines.push(Line::from(""));
    }

    let width = inner_area.width.max(1) as usize;
    let mut rendered_lines = 0;
    for line in &lines {
        let line_len: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
        if line_len == 0 {
            rendered_lines += 1;
        } else {
            rendered_lines += (line_len + width - 1) / width;
        }
    }

    let visible_lines = inner_area.height as usize;
    let max_scroll = rendered_lines.saturating_sub(visible_lines);
    app.max_scroll.set(max_scroll);
    let actual_scroll = app.scroll.min(max_scroll) as u16;

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((actual_scroll, 0));

    f.render_widget(p, area);

    // Barra de rolagem visual à direita
    let mut scrollbar_state = ScrollbarState::new(max_scroll).position(actual_scroll as usize);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
        .style(Style::default().fg(Color::Rgb(255, 60, 140))); // Rosa Neon Boitatá

    f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.is_generating {
        Color::Yellow
    } else {
        Color::Rgb(0, 230, 255) // Neon Cyan
    };

    let title = if app.is_generating {
        " ⏳ Tuitatá gerando resposta... ".to_string()
    } else if let Some(att) = &app.pending_image {
        format!(" Pergunta 📎 [Anexada: {}] (Ctrl+X remove) ", att.filename)
    } else {
        " Pergunta (Enter envia) ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let inner = block.inner(area);
    let inner_width = inner.width as usize;

    if app.input.is_empty() && !app.is_generating {
        let display_text = Span::styled(
            "Digite uma dúvida ou instrução para a IA...",
            Style::default().fg(Color::DarkGray),
        );
        let p = Paragraph::new(display_text).block(block);
        f.render_widget(p, area);
        f.set_cursor_position((inner.x, inner.y));
        return;
    }

    let chars: Vec<char> = app.input.chars().collect();
    let char_count = chars.len();
    let cursor_pos = app.cursor_pos.min(char_count);

    let scroll_offset = if inner_width == 0 {
        0
    } else if cursor_pos >= inner_width {
        cursor_pos - inner_width + 1
    } else {
        0
    };

    let end_idx = (scroll_offset + inner_width).min(char_count);
    let visible_slice: String = if scroll_offset < char_count {
        chars[scroll_offset..end_idx].iter().collect()
    } else {
        String::new()
    };

    let display_text = Span::styled(visible_slice, Style::default().fg(Color::White));
    let p = Paragraph::new(display_text).block(block);
    f.render_widget(p, area);

    // Posiciona o cursor visual na barra de entrada
    if !app.is_generating {
        let cursor_x = inner.x + (cursor_pos.saturating_sub(scroll_offset) as u16);
        let max_x = inner.x + inner.width.saturating_sub(1);
        f.set_cursor_position((cursor_x.min(max_x), inner.y));
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let status_line = if let Some((msg, instant)) = &app.status {
        if instant.elapsed().as_secs() < 5 {
            Line::from(vec![Span::styled(
                format!(" 📢 {}", msg),
                Style::default()
                    .fg(Color::Rgb(100, 255, 100))
                    .add_modifier(Modifier::BOLD),
            )])
        } else {
            Line::from(Span::styled(
                " Tuitatá v0.1.0 │ Padrão GOST / Baquara",
                Style::default().fg(Color::DarkGray),
            ))
        }
    } else {
        Line::from(Span::styled(
            " Tuitatá v0.1.0 │ Padrão GOST / Baquara",
            Style::default().fg(Color::DarkGray),
        ))
    };

    let p = Paragraph::new(status_line);
    f.render_widget(p, area);
}
