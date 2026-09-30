# GOST — Manifesto de Arquitetura do Tuitatá (`tt`)

## 1. Identificação do Projeto
- **Nome:** Tuitatá (`tuitata` / `tt`)
- **Ecossistema:** Baquará
- **Perfil:** Lumen (`profiles/lumen.md`)
- **Linguagem:** Rust (2021)
- **Escopo:** Cliente TUI de alta velocidade para o Google Gemini com suporte a streaming, histórico local e exportação Obsidian.

## 2. Princípios Arquiteturais (Lumen Profile)
- **Zero Overhead:** Binário único compilado em release sem dependências em runtime.
- **Isolamento de Domínio:** Divisão estrita entre a camada de apresentação (`ui.rs`), orquestração (`app.rs`), clientes de rede (`gemini.rs`) e persistência local (`history.rs`).
- **Resiliência de Rede:** Timeout e fallback graceful para erros de API, limites de cota e ausência de conexão.
- **XDG Compliance:** Armazenamento em `~/.config/tuitata` e `~/.local/share/tuitata`.
