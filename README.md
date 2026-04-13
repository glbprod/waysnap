# waysnap

Window snapping helper for **labwc** (Wayland compositor).

Injects `SnapToEdge` keybindings into `~/.config/labwc/rc.xml` and reloads
labwc via SIGHUP — no daemon, no IPC, no runtime dependency.

---

## How it works

labwc exposes two mechanisms relevant to snapping:

1. **`SnapToEdge` action** — native labwc action that snaps the active window
   to a screen edge or corner. Declared as a keybinding in `rc.xml`.
2. **SIGHUP** — sending SIGHUP to the labwc process (identified by `$LABWC_PID`)
   triggers an immediate reload of `rc.xml`, equivalent to `labwc -r`.

`waysnap install` combines both: it patches `rc.xml` to add the keybindings,
then sends SIGHUP to apply them instantly. No restart required.

```
waysnap install
     │
     ├─► reads ~/.config/labwc/rc.xml
     ├─► backs up to rc.xml.waysnap.bak
     ├─► patches rc.xml (handles 4 cases, see below)
     └─► kill($LABWC_PID, SIGHUP)  →  labwc reloads
```

---

## Keybindings installed

| Shortcut | Action |
|---|---|
| `Super + Left` | Snap to left half |
| `Super + Right` | Snap to right half |
| `Super + Up` | Snap to top half |
| `Super + Down` | Snap to bottom half |
| `Super + F` | Toggle fullscreen (maximize) |
| `Super + KP_7` | Snap to top-left quarter |
| `Super + KP_9` | Snap to top-right quarter |
| `Super + KP_1` | Snap to bottom-left quarter |
| `Super + KP_3` | Snap to bottom-right quarter |

The modifier key defaults to `W` (Super/Logo). Use `--modifier A` for Alt, etc.

Multi-monitor is handled automatically by labwc: snapping always applies to
the output where the active window currently resides.

---

## rc.xml patch logic

`waysnap` handles four states of `rc.xml`:

| State | Action |
|---|---|
| File does not exist | Creates a minimal valid `rc.xml` with the snippet |
| Already contains `<!-- waysnap: -->` | Replaces the existing block in-place |
| Root tag is self-closing (`<openbox_config ... />`) | Expands it to a proper open/close pair, then inserts snippet |
| Normal open/close root tag | Inserts snippet before `</labwc_config>` or `</openbox_config>` |

Both `<labwc_config>` (native labwc format) and `<openbox_config>` (OpenBox
compatibility format) are supported as root elements.

A backup is always written to `rc.xml.waysnap.bak` before any modification.

---

## XML snippet injected

```xml
<!-- waysnap: window snapping keybindings (modifier=W) -->
<keyboard>
  <!-- Half-screen snapping -->
  <keybind key="W-Left">
    <action name="SnapToEdge" direction="left"/>
  </keybind>
  <keybind key="W-Right">
    <action name="SnapToEdge" direction="right"/>
  </keybind>
  <keybind key="W-Up">
    <action name="SnapToEdge" direction="up"/>
  </keybind>
  <keybind key="W-Down">
    <action name="SnapToEdge" direction="down"/>
  </keybind>

  <!-- Fullscreen toggle -->
  <keybind key="W-f">
    <action name="ToggleMaximize"/>
  </keybind>

  <!-- Quarter snapping -->
  <keybind key="W-KP_7">
    <action name="SnapToEdge" direction="up-left"/>
  </keybind>
  <keybind key="W-KP_9">
    <action name="SnapToEdge" direction="up-right"/>
  </keybind>
  <keybind key="W-KP_1">
    <action name="SnapToEdge" direction="down-left"/>
  </keybind>
  <keybind key="W-KP_3">
    <action name="SnapToEdge" direction="down-right"/>
  </keybind>
</keyboard>
<!-- end waysnap -->
```

---

## Usage

```sh
# Install keybindings and reload labwc immediately
waysnap install

# Use Alt instead of Super
waysnap install --modifier A

# Print the XML snippet without touching any file
waysnap show-config

# Reload labwc without modifying rc.xml
waysnap reload
```

---

## Build & install

```sh
# Debug build
make build

# Optimized native build
make release

# Install to ~/.local/bin
make install

# Cross-compile for aarch64 (Raspberry Pi 64-bit)
# Requires: rustup target add aarch64-unknown-linux-gnu
#           apt install gcc-aarch64-linux-gnu
make cross
make install-cross

# Run unit tests
make test
```

---

## Requirements

| Requirement | Details |
|---|---|
| labwc | ≥ 0.6 (SnapToEdge was introduced around 0.6) |
| `$LABWC_PID` | Set automatically by labwc at startup |
| `$HOME` or `$XDG_CONFIG_HOME` | Used to locate `rc.xml` |
| Rust | 1.65+ (edition 2021) |

No runtime dependencies. The binary links only against libc (`kill` syscall).

---

## Project layout

```
waysnap/
├── Cargo.toml        — clap 4 only; release profile optimized for size
├── Makefile          — build / install / cross-compile targets
├── rc.xml.demo       — annotated example rc.xml for reference
└── src/
    ├── main.rs       — CLI (install / show-config / reload)
    └── labwc_ipc.rs  — rc.xml patching logic + SIGHUP
```

---

## About

This project was entirely written by **Claude Sonnet 4.6** via
**[OpenCode](https://opencode.ai)**, an AI-powered coding agent for the
terminal. From architecture decisions to edge-case handling and unit tests,
every line of Rust was generated through a conversational session — no manual
coding involved.

**Lead developer:** glbprod — architecture, requirements & prompt engineering
**Code generation:** Claude Sonnet 4.6 via OpenCode
