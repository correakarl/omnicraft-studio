#!/usr/bin/env bash
# Script lanzador rápido para Omnicraft Launcher Studio v2.0 (Desarrollo y Escritorio)

cd "$(dirname "$0")"

export PATH="$HOME/.cargo/bin:$PATH"

echo "=========================================================="
echo "🚀 Iniciando Omnicraft Launcher Studio v2.0 (Tauri + Svelte)"
echo "=========================================================="

# Comprobar si pnpm está disponible
if command -v pnpm &> /dev/null; then
    pnpm tauri dev
else
    npm run tauri dev
fi
