# Changelog — Tuitatá (`tt`)

Todas as modificações notáveis deste projeto serão documentadas neste arquivo.
O formato é baseado no [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e este projeto segue [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [0.2.0-dev.1] - 2026-09-30

### Adicionado
- **Visão Computacional Multimodal**:
  - Atalho `<Ctrl+V>` / `<Alt+V>` para colar imagens diretamente da área de transferência do X11 (`arboard` + `image`).
  - Atalho `<Ctrl+I>` e comandos `/image <caminho>` e `/imagem <caminho>` para anexar arquivos locais.
  - Atalho `<Ctrl+X>` e comando `/noimage` para descartar imagem pendente.
  - Envio de dados `inlineData` (base64) compatível com Gemini 3.8 Flash e Ollama Vision.
  - Indicador visual dinâmico na barra de entrada e badge `📎 [Imagem]` no histórico e na exportação do Obsidian.
- **Geração de Imagens (Text-to-Image)**:
  - Comandos `/imagine <prompt>` e `/gerar <prompt>` conectados à API do Gemini (`gemini-3.1-flash-image` / Nano Banana).
  - Salvamento automático em `~/obsidian/01_inbox/imagens/{data}_{slug}.jpg`.
  - Disparo em segundo plano do visualizador `nsxiv` para renderização imediata na tela.
- **Estabilidade do Event Loop**:
  - Canal determinístico de finalização de stream eliminando race condition na liberação da barra de status.

## [0.1.0-dev.1] - 2026-09-30

### Adicionado
- Fundação do repositório no padrão GOST (Lumen Profile).
- Cliente TUI interativo em Rust (`ratatui` + `crossterm`).
- Streaming em tempo real com a API do Google Gemini (`gemini-2.5-flash` / `gemini-2.5-pro`).
- Exportação direta de conversas para o Obsidian (`~/obsidian/01_inbox/` ou `94_diario/`).
- Cópia limpa de blocos de código para o clipboard com `Alt+y`.
- Atalhos `tuitata` e binário curto `tt`.
