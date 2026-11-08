#!/usr/bin/env bash
set -e

echo "==> Building clipsync (release mode)..."
cargo build --release

echo "==> Installing binaries to ~/.local/bin/..."
mkdir -p "$HOME/.local/bin"
install -Dm755 target/release/clipsync-server "$HOME/.local/bin/clipsync-server"
install -Dm755 target/release/clipsync-client "$HOME/.local/bin/clipsync-client"

echo "==> Installing systemd user units to ~/.config/systemd/user/..."
mkdir -p "$HOME/.config/systemd/user"
install -Dm644 systemd/clipsync-server.service "$HOME/.config/systemd/user/clipsync-server.service"
install -Dm644 systemd/clipsync-client.service "$HOME/.config/systemd/user/clipsync-client.service"

echo "==> Reloading systemd user daemon..."
systemctl --user daemon-reload

echo ""
echo "Successfully installed clipsync!"
echo ""
echo "To start the server (e.g. on host PC):"
echo "  systemctl --user enable --now clipsync-server"
echo ""
echo "To start the client:"
echo "  systemctl --user enable --now clipsync-client"
echo ""
echo "To configure client for other devices, create ~/.config/clipsync/client.env:"
echo "  CLIPSYNC_SERVER=192.168.1.103:9090"
echo "  CLIPSYNC_NAME=laptop"
