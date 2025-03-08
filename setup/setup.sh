#!/bin/bash

set -e

REPO="https://github.com/gen2-acidking/remo/releases/latest/download"
BINARY="remo"
CONFIG_DIR="$HOME/.config/remo"
CONFIG_FILE="$CONFIG_DIR/config.lua"
ALIAS_NAME="remo"

echo "🚀 Installing Remote Governor from $REPO..."

# /usr/local/bin exists
if [ ! -d "/usr/local/bin" ]; then
    echo "❌ Error: /usr/local/bin does not exist. Create it or adjust the script."
    exit 1
fi

# dl binary latest
echo "➡️ Downloading binary..."
curl -L -o "/usr/local/bin/$BINARY" "$REPO/remo"
chmod +x "/usr/local/bin/$BINARY"

# create config dir
echo "➡️ Creating config directory at $CONFIG_DIR..."
mkdir -p "$CONFIG_DIR"

# download conf
echo "➡️ Downloading config file..."
curl -L -o "$CONFIG_FILE" "$REPO/config.lua"
chmod 600 "$CONFIG_FILE"

# alias to shell
if ! grep -q "alias $ALIAS_NAME=" "$HOME/.bashrc" "$HOME/.zshrc"; then
    echo "➡️ Adding alias to shell..."
    echo "alias $ALIAS_NAME='/usr/local/bin/$BINARY'" >> "$HOME/.bashrc"
    echo "alias $ALIAS_NAME='/usr/local/bin/$BINARY'" >> "$HOME/.zshrc"
fi

# source shell config
if [[ "$SHELL" == *"zsh"* ]]; then
    source "$HOME/.zshrc"
else
    source "$HOME/.bashrc"
fi

echo "✅ Installation complete!"
echo "➡️ Run 'remo setup' for configuration."

