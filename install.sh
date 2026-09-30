#!/usr/bin/env bash
# Tuitatá (tt) - Script de compilação e instalação
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
cd "$SCRIPT_DIR"

echo "==> [Tuitatá] Compilando binário otimizado em release..."
cargo build --release

echo "==> [Tuitatá] Instalando binário em ~/.local/bin..."
mkdir -p ~/.local/bin
install -m 755 target/release/tuitata ~/.local/bin/tuitata

# Criar atalhos curtos tt e ttt
ln -sf ~/.local/bin/tuitata ~/.local/bin/tt
ln -sf ~/.local/bin/tuitata ~/.local/bin/ttt

echo "==> [Tuitatá] Instalação concluída com sucesso! (Comandos: tuitata, tt, ttt)"
~/.local/bin/tt "diga 'Tuitatá pronto!' em 2 palavras"
