#!/usr/bin/env bash
set -e

echo "==> Stopping systemd services..."
systemctl --user stop clipsync-client.service 2>/dev/null || true
systemctl --user stop clipsync-server.service 2>/dev/null || true
systemctl --user disable clipsync-client.service 2>/dev/null || true
systemctl --user disable clipsync-server.service 2>/dev/null || true

echo "==> Removing installed files..."
rm -f "$HOME/.local/bin/clipsync-server"
rm -f "$HOME/.local/bin/clipsync-client"
rm -f "$HOME/.config/systemd/user/clipsync-server.service"
rm -f "$HOME/.config/systemd/user/clipsync-client.service"

systemctl --user daemon-reload

echo "clipsync has been completely uninstalled."
