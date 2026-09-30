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
- **Alternador de Modelos:** Atalhos `<Ctrl+P>`, `<Tab>` ou comando `/model` alternam dinamicamente entre **⚡ Gemini Flash** (ultrarrápido) e **🧠 Gemini Pro** (raciocínio profundo).
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
| `<Ctrl + U>` / `<Ctrl + D>` | Rola meia página para cima ou para baixo (ideal para tablets) |
| `<Ctrl + K>` / `<Ctrl + J>` | Rola uma linha para cima ou para baixo (`Alt+↑` / `Alt+↓`) |
| `PageUp / PageDown` | Rola blocos do histórico da conversa |
| `/top` ou `/inicio` | Digite no prompt para saltar imediatamente ao início da conversa |
| `/bottom` ou `/fim` | Digite no prompt para saltar imediatamente ao final da conversa |
| `Esc` ou `Ctrl + C` | Encerra o Tuitatá |

### 3. Modo Pergunta Rápida no Terminal (CLI)
Você também pode disparar perguntas pontuais diretamente sem abrir o TUI completo:
```bash
tt "explique o que é a lei de Amdahl em 3 linhas"
```

---

## ⚙️ Configuração

O Tuitatá busca automaticamente a chave `GEMINI_API_KEY`:
1. Na variável de ambiente `GEMINI_API_KEY`;
2. No arquivo `~/.env`.

---

## 📜 Conformidade GOST
Este repositório adota e cumpre as diretrizes da especificação **GOST** (Lumen Profile).
Consulte `.gost/GOST.md` para detalhes arquiteturais.
