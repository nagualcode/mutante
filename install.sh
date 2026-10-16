#!/bin/bash
# Mutante macOS Quick Installer
# Jardim Sonoro (2026)

set -e

echo "=========================================================="
echo "Installing Mutante (Mu-Tron Bi-Phase Physical Emulation)"
echo "Jardim Sonoro"
echo "=========================================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DEST_VST3="$HOME/Library/Audio/Plug-Ins/VST3"
DEST_CLAP="$HOME/Library/Audio/Plug-Ins/CLAP"
DEST_AU="$HOME/Library/Audio/Plug-Ins/Components"
DEST_PRESETS="$HOME/Library/Audio/Presets/Jardim Sonoro/Mutante"

mkdir -p "$DEST_VST3" "$DEST_CLAP" "$DEST_AU" "$DEST_PRESETS"

if [ -d "$SCRIPT_DIR/Mutante.vst3" ]; then
    echo "-> Installing VST3 to $DEST_VST3/Mutante.vst3"
    rm -rf "$DEST_VST3/Mutante.vst3"
    cp -R "$SCRIPT_DIR/Mutante.vst3" "$DEST_VST3/"
    xattr -cr "$DEST_VST3/Mutante.vst3" 2>/dev/null || true
fi

if [ -d "$SCRIPT_DIR/Mutante.clap" ]; then
    echo "-> Installing CLAP to $DEST_CLAP/Mutante.clap"
    rm -rf "$DEST_CLAP/Mutante.clap"
    cp -R "$SCRIPT_DIR/Mutante.clap" "$DEST_CLAP/"
    xattr -cr "$DEST_CLAP/Mutante.clap" 2>/dev/null || true
fi

if [ -d "$SCRIPT_DIR/Mutante.component" ]; then
    echo "-> Installing AU (Component) to $DEST_AU/Mutante.component"
    rm -rf "$DEST_AU/Mutante.component"
    cp -R "$SCRIPT_DIR/Mutante.component" "$DEST_AU/"
    xattr -cr "$DEST_AU/Mutante.component" 2>/dev/null || true
fi

if [ -d "$SCRIPT_DIR/Presets" ]; then
    echo "-> Installing Presets to $DEST_PRESETS"
    cp -R "$SCRIPT_DIR/Presets/"* "$DEST_PRESETS/" 2>/dev/null || true
fi

echo "=========================================================="
echo "Installation complete! Restart your DAW to rescan."
echo "Mutante is located under the 'Jardim Sonoro' vendor folder."
echo "=========================================================="
