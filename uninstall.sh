#!/bin/bash
# Mutante macOS uninstaller
# Jardim Sonoro (2026)
#
# Removes the Mutante plug-ins and factory presets from the per-user and
# (when run with sudo) system plug-in folders. Your own saved presets in
# the DAW are left alone unless you pass --presets.

set -e

REMOVE_PRESETS=0
[ "$1" = "--presets" ] && REMOVE_PRESETS=1

echo "=========================================================="
echo "Uninstalling Mutante (Mu-Tron Bi-Phase emulation)"
echo "=========================================================="

remove() {
    if [ -e "$1" ]; then
        echo "-> Removing $1"
        rm -rf "$1"
    fi
}

for ROOT in "$HOME/Library/Audio/Plug-Ins" "/Library/Audio/Plug-Ins"; do
    if [ "$ROOT" = "/Library/Audio/Plug-Ins" ] && [ ! -w "$ROOT/VST3" ] 2>/dev/null; then
        if [ -e "$ROOT/VST3/Mutante.vst3" ] || [ -e "$ROOT/CLAP/Mutante.clap" ] || [ -e "$ROOT/Components/Mutante.component" ]; then
            echo "!! System-wide copies found in $ROOT; re-run with sudo to remove them."
        fi
        continue
    fi
    remove "$ROOT/VST3/Mutante.vst3"
    remove "$ROOT/CLAP/Mutante.clap"
    remove "$ROOT/Components/Mutante.component"
done

if [ "$REMOVE_PRESETS" = "1" ]; then
    remove "$HOME/Library/Audio/Presets/Jardim Sonoro/Mutante"
fi

# Make Logic / GarageBand / Live forget the Audio Unit.
killall -9 AudioComponentRegistrar 2>/dev/null || true

echo "=========================================================="
echo "Done. Restart your DAW so it rescans its plug-ins."
echo "=========================================================="
