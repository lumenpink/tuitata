# 🔥 Tuitatá (`tt`)
> **Cliente TUI para Google Gemini em Rust com streaming, estética Boitatá e exportação para o Obsidian.**  
> Desenvolvido no padrão **GOST** (Lumen Profile) para o ecossistema Baquara.

---

## 🗺️ Visão Geral

O **Tuitatá** (`tt` / `tuitata`) é um cliente de terminal interativo para interagir com os modelos mais avançados do Google Gemini (`gemini-3.8-flash` e `gemini-3.1-pro-preview`):

- **Binários:** `tuitata`, `tt`, `ttt`.
- **Streaming Instantâneo:** Respostas geradas e exibidas em tempo real via Server-Sent Events (SSE).
- **Exportação para o Obsidian:** Atalho `<Ctrl+S>` salva imediatamente a conversa formatada com metadados YAML dentro de `~/obsidian/01_inbox/`.
- **Cópia Rápida de Código:** Atalho `<Alt+Y>` copia o último bloco de código para a área de transferência do sistema (`Ctrl+V`).
- **Alternador de Modelos:** Atalhos `<Ctrl+P>`, `<Tab>` ou comando `/model` alternam dinamicamente entre **⚡ Gemini Flash** (ultrarrápido), **🧠 Gemini Pro** (raciocínio avançado) e **🦙 Ollama Local** (`qwen2.5-coder:3b` offline, sem consumir créditos).
- **Provedor Local Ollama:** Execução 100% offline via API local (`localhost:11434`) com modelos de código (`qwen2.5-coder`) servindo como pré-agente.
- **Barra de Rolagem Visual:** Indicador vertical à direita exibindo trilho, posição atual e proporção do histórico de mensagens em tempo real.
- **Estética Boitatá / Cyberpunk:** Cores de alto contraste em Rosa Elétrico (`#ff3c8c`), Amarelo Ácido (`#ffe664`) e Ciano Neon (`#00e6ff`).

---

## 🚀 Instalação e Compilação

```bash
cd /home/lumen/src/tuitata
./install.sh
```

O instalador compilará o binário otimizado em release e criará os links em `~/.local/bin/`:
- `tuitata`
- `tt`
- `ttt`

### Compilação Estática e Otimização Soberana (GOST 1.5.0)
- **Perfil de Release Otimizado:** `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` e `strip = true`.
- **Linking Estático musl:** Suporte hermético com `target-feature=+crt-static` para distribuição `static-pie` universal sem dependências externas.

---

## 💻 Como Usar

### 1. Abrir a Interface TUI Interativa
Basta digitar:
```bash
tt
# ou
tuitata
```

### 2. Atalhos de Teclado no TUI

| Tecla | Ação |
| :--- | :--- |
| `Enter` | Envia a mensagem para o Gemini |
| `← / →` | Move o cursor na barra de entrada de texto (com rolagem horizontal automática) |
| `Backspace / Del` | Apaga caractere anterior ou posterior ao cursor |
| `<Ctrl + P>` / `<Tab>` | Alterna entre os modelos de IA (ou digite `/model` / `/m`) |
| `<Ctrl + S>` | Exporta a conversa para `~/obsidian/01_inbox/` em Markdown |
| `<Alt + Y>` | Copia o último bloco de código gerado para o clipboard |
| `<Ctrl + W>` | Grava o arquivo proposto pela IA diretamente no disco |
| `<Ctrl + F>` | Inicia busca textual no histórico ou salta para a próxima ocorrência |
| `<Ctrl + U>` / `<Ctrl + D>` | Rola meia página para cima ou para baixo (pausa o autoscroll durante streaming) |
| `<Ctrl + K>` / `<Ctrl + J>` | Rola uma linha para cima ou para baixo (`Alt+↑` / `Alt+↓`) |
| `PageUp / PageDown` | Rola blocos do histórico da conversa |
| `/top` ou `/inicio` | Digite no prompt para saltar imediatamente ao início da conversa |
| `/bottom` ou `/fim` | Digite no prompt para saltar imediatamente ao final da conversa |
| `/find <termo>` ou `/<termo>` | Busca termo no histórico com destaque colorido e salto imediato |
| `/clear_find` | Remove o realce e encerra o filtro de busca |
| `/apply` ou `/write` | Aplica/grava o arquivo sugerido pela IA no disco |
| `/save <caminho>` | Grava o último bloco de código no caminho especificado |
| `Esc` ou `Ctrl + C` | Encerra o Tuitatá |

### 3. Análise e Interação com Arquivos Locais
Você pode referenciar arquivos locais diretamente no prompt de duas formas:
- **Menção direta com `@`**:
  ```text
  analise o código em @src/main.rs e explique os atalhos
  ```
- **Comando `/file` (ou `/ler`)**:
  ```text
  /file Cargo.toml quais dependências estão desatualizadas?
  ```
O Tuitatá detecta a sintaxe do arquivo, valida o tamanho (até 500 KB) e injeta o bloco de código formatado automaticamente no prompt para a IA.

### 4. Gravação de Arquivos no Disco (Estilo Antigravity / Copilot)
Quando o modelo propõe a criação ou modificação de arquivos de código:
- Pressione `<Ctrl + W>` ou digite `/apply` (ou `/write`) para persistir diretamente o arquivo proposto pela IA no caminho correto.
- Ou use `/save <caminho>` para direcionar o conteúdo para um local customizado.
- O Tuitatá cria automaticamente diretórios intermediários necessários e emite confirmação de gravação na barra de status.

### 5. Modo Pergunta Rápida no Terminal (CLI)
Você também pode disparar perguntas pontuais diretamente com suporte a arquivos sem abrir a TUI:
```bash
tt "explique o que é a lei de Amdahl em 3 linhas"
tt @Cargo.toml "resuma as dependências do projeto"
tt /file src/app.rs "encontre possíveis vazamentos de memória"

# Execução offline via Ollama local (zero consumo de créditos)
tt --ollama @src/main.rs "analise o código com o qwen-coder"
```

---

## ⚙️ Configuração

O Tuitatá busca automaticamente suas configurações:
1. **Google Gemini:** Chave em `GEMINI_API_KEY` (ambiente ou `~/.env`);
2. **Ollama Local:** URL padrão `http://localhost:11434` (configurável via `OLLAMA_URL`) e modelo padrão `qwen2.5-coder:3b` (configurável via `OLLAMA_MODEL`);
3. **Obsidian:** Exportação automática em `~/obsidian/01_inbox/`.

---

## 📜 Conformidade GOST
Este repositório adota e cumpre as diretrizes da especificação **GOST** (Lumen Profile).
Consulte `.gost/GOST.md` para detalhes arquiteturais.
