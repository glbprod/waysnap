# Reddit posts — waysnap launch

---

## r/labwc

**Title:** `waysnap — Super+Arrow window snapping for labwc, one command to install`

```
If you've been missing Ubuntu-style window snapping on labwc, I wrote a small
tool that sets it up in one command:

    waysnap install

It injects SnapToEdge keybindings into ~/.config/labwc/rc.xml and reloads
labwc via SIGHUP. No daemon, no IPC socket, no Wayland protocol magic —
just rc.xml + SIGHUP, the way labwc intends it.

Keybindings installed:
  Super+Left/Right/Up/Down  →  half-screen snapping
  Super+F                   →  toggle fullscreen
  Super+KP_7/9/1/3          →  quarter snapping (numpad)

Written in Rust, single dependency (clap), prebuilt aarch64 binary available
for Raspberry Pi users.

https://github.com/glbprod/waysnap

Feedback welcome — especially if you're on a multi-monitor setup.
```

---

## r/raspberry_pi

**Title:** `Super+Arrow window snapping on Raspbian + labwc (Wayland) — one command`

```
Been running labwc on my Pi and missed having proper window snapping.
Built a small Rust tool called waysnap that adds it in one shot:

    curl -LO https://github.com/glbprod/waysnap/releases/latest/download/waysnap-aarch64
    chmod +x waysnap-aarch64 && mv waysnap-aarch64 ~/.local/bin/waysnap
    waysnap install

That's it. Super+Arrow keys snap windows to halves, Super+KP corners snap
to quarters. Tested on Raspbian with labwc 0.9.2.

Prebuilt aarch64 binary, no compilation needed.

https://github.com/glbprod/waysnap
```

---

## r/wayland

**Title:** `waysnap — window snapping for labwc via SnapToEdge + SIGHUP, no protocol needed`

```
Quick share: I wrote waysnap, a CLI tool that sets up window snapping on labwc.

The interesting part from a Wayland perspective: there's no ext-foreign-toplevel
or wlr-layer-shell involved. labwc's native SnapToEdge action handles everything
— the tool just patches rc.xml and sends SIGHUP to reload the config.

Turns out for this use case, the compositor's own config system is the right
layer to work at, not the Wayland protocol layer.

Rust, ~450KB aarch64 binary, one dependency (clap).

https://github.com/glbprod/waysnap
```

---

## r/rust

**Title:** `waysnap — tiny Rust CLI that patches XML config and sends SIGHUP, for labwc window snapping`

```
Small weekend project: a CLI tool that adds window snapping to the labwc
Wayland compositor by patching its XML config file and reloading it via SIGHUP.

Nothing fancy technically, but a few things worth sharing:

- No libc crate — just an extern "C" { fn kill(...) } declaration for the
  SIGHUP call. Turns out that's all you need.
- XML patching without a parser — handles 4 structural cases (self-closing
  root tag, existing block replacement, missing file, normal insert) with
  plain string operations. Robust enough for a config file you control.
- Release profile: opt-level="z", lto=true, strip=true → 453KB aarch64 binary.
- Single dependency: clap 4.

https://github.com/glbprod/waysnap
```
