# sc - Sway Screen Configuration

A simple GUI tool for managing monitor layout in [Sway](https://swaywm.org/).

## Features

- Visual monitor layout with drag-and-drop repositioning
- Edge snapping when placing monitors
- Resolution selection per monitor
- Extend / Mirror mode toggle
- Auto-detects monitor connect/disconnect via Sway IPC events
- Bottom-aligned monitor layout

## Screenshot

```
+--------------------------------------------------+
|          (o) Extend  ( ) Mirror                   |
+--------------------------------------------------+
|          +------+  +-----------+                  |
|          | eDP-1|  |   DP-1    |                  |
|          +------+  +-----------+                  |
+--------------------------------------------------+
|           DP-1 (Dell U2715D)                      |
|       [1920x1080 @ 60Hz  v]  [Apply]              |
+--------------------------------------------------+
```

## Building

Requires Rust 1.75+.

```sh
cargo build --release
```

The binary is at `target/release/sc`.

## Installation

```sh
cp target/release/sc ~/.local/bin/
```

To make the window float in Sway, add to `~/.config/sway/config`:

```
for_window [title="sc - Screen Config"] floating enable
```

## Usage

Launch `sc` from a terminal or application launcher. Click a monitor in the
canvas to select it, drag to reposition. Select a resolution from the dropdown
and click Apply to send the configuration to Sway.

Mirror mode sets all outputs to position 0,0 with a common resolution.

## Dependencies

- [iced](https://iced.rs/) - GUI framework
- [swayipc](https://github.com/JayceFayne/swayipc-rs) - Sway IPC client

## License

MIT
