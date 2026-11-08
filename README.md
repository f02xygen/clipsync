# clipsync

A lightweight, zero-dependency, cross-device clipboard synchronization daemon for Linux (Wayland).

Syncs text and screenshots seamlessly between your desktop and laptop over a local network.

---

## Features

- 🚀 **Zero External Dependencies** — Built entirely using pure Rust standard library (`std` only). No `tokio`, no `serde`, no third-party crates.
- 🖼️ **Text & Image Sync** — Synchronizes both UTF-8 text and raw images (`image/png` screenshots).
- ⚡ **Wayland Native** — Interacts cleanly with Wayland compositors (Niri, KDE Plasma, Hyprland, Sway, GNOME, etc.) via `wl-clipboard`.
- 🛡️ **Echo & Loop Prevention** — Fast 64-bit hashing (`DefaultHasher`) stops infinite broadcast ping-pong loops dead in their tracks.
- 🏷️ **Device Identification** — Shows which computer sent each item, defaulting to system `/etc/hostname` or custom names.
- 🔄 **Self-Healing** — Reconnects automatically on network drops when managed by `systemd --user`.

---

## Architecture

```
                       +-------------------+
                       |  clipsync-server  |
                       |  (TCP Broadcast)  |
                       +---------+---------+
                                 |
              +------------------+------------------+
              |                                     |
              v                                     v
    +-------------------+                 +-------------------+
    |  clipsync-client  |                 |  clipsync-client  |
    |     (PC / Niri)   |                 |   (Laptop / KDE)  |
    +---------+---------+                 +---------+---------+
              |                                     |
              v                                     v
      Wayland Clipboard                     Wayland Clipboard
        (wl-clipboard)                        (wl-clipboard)
```

---

## Requirements

- **Linux** with a Wayland compositor
- **[Rust & Cargo](https://rustup.rs/)** (1.80+ recommended)
- **`wl-clipboard`** (`wl-copy` and `wl-paste`):
  - Arch Linux: `sudo pacman -S wl-clipboard`
  - Fedora: `sudo dnf install wl-clipboard`
  - Ubuntu / Debian: `sudo apt install wl-clipboard`

---

## Quick Start

### 1. Clone & Install

```bash
git clone https://github.com/f02xygen/clipsync.git
cd clipsync
./install.sh
```

The install script compiles release binaries to `~/.local/bin/` and registers `systemd --user` units.

---

### 2. Run the Server (e.g. on your PC or Home Server)

Enable and start the server service:

```bash
systemctl --user enable --now clipsync-server
```

*(By default, the server binds to `0.0.0.0:9090`)*.

> [!NOTE]
> If you have a firewall enabled (like UFW), allow incoming TCP traffic on port 9090:
> ```bash
> sudo ufw allow 9090/tcp
> ```

---

### 3. Run the Client

#### On the machine running the server (PC):
The client connects to `127.0.0.1:9090` by default:

```bash
systemctl --user enable --now clipsync-client
```

#### On other machines (e.g. Laptop):
Create a configuration file `~/.config/clipsync/client.env` pointing to your server's LAN IP:

```bash
mkdir -p ~/.config/clipsync
cat <<EOF > ~/.config/clipsync/client.env
# Replace with your server's LAN IP and port
CLIPSYNC_SERVER=192.168.1.100:9090
CLIPSYNC_NAME=laptop
EOF
```

Then start the client:

```bash
systemctl --user enable --now clipsync-client
```

---

## Manual / CLI Usage

You can also run the binaries directly without systemd:

```bash
# Start server manually:
clipsync-server [bind_address]
# Example:
clipsync-server 0.0.0.0:9090

# Start client manually:
clipsync-client [server_address] [device_name]
# Example:
clipsync-client 192.168.1.103:9090 laptop
```

---

## Checking Logs

View live logs from the background services:

```bash
# Client logs:
journalctl --user -u clipsync-client -f

# Server logs:
journalctl --user -u clipsync-server -f
```

---

## License

MIT OR Apache-2.0
