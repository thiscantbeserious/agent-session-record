#!/usr/bin/env bash
set -e

echo "=== Agent Session Recorder Uninstaller ==="
echo

# Remove binary
INSTALL_DIR="$HOME/.local/bin"
if [ -f "$INSTALL_DIR/asr" ]; then
    rm "$INSTALL_DIR/asr"
    echo "Removed binary: $INSTALL_DIR/asr"
else
    echo "Binary not found at: $INSTALL_DIR/asr"
fi

# Remove config directory (ask first)
CONFIG_DIR="$HOME/.config/asr"
if [ -d "$CONFIG_DIR" ]; then
    read -p "Remove config directory ($CONFIG_DIR)? [y/N]: " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        echo "Removed config directory"
    else
        echo "Kept config directory"
    fi
fi

# Remove session directory (ask first)
SESSION_DIR="$HOME/recorded_agent_sessions"
if [ -d "$SESSION_DIR" ]; then
    read -p "Remove session recordings ($SESSION_DIR)? [y/N]: " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$SESSION_DIR"
        echo "Removed session directory"
    else
        echo "Kept session directory"
    fi
fi

# Remove skills
echo
echo "Removing skills..."
if command -v asr &>/dev/null; then
    asr skills uninstall
else
    # Fallback: manually remove skill files if asr is not available
    echo "asr not found in PATH, removing skills manually..."
    for dir in "$HOME/.claude/commands" "$HOME/.codex/commands" "$HOME/.gemini/commands"; do
        for skill in "asr-analyze.md" "asr-review.md"; do
            if [ -f "$dir/$skill" ] || [ -L "$dir/$skill" ]; then
                rm "$dir/$skill"
                echo "  Removed: $dir/$skill"
            fi
        done
    done
fi

# Note about shell integration
echo
echo "Note: Shell integration line in .zshrc/.bashrc was NOT removed."
echo "You can manually remove the 'Agent Session Recorder' section if desired."

echo
echo "=== Uninstallation Complete ==="
