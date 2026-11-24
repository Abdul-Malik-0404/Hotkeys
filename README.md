# Hotkeys — Keybind & Media Shortcut Manager (Windows & macOS)

A lightweight Rust desktop application for mapping global keyboard shortcuts (`Ctrl`, `Alt`, `Shift`, `Win`/`Cmd` + Key) to custom key combinations, media playback keys, or volume controls on **Windows** and **macOS**.

## Features

- **Interactive Keybind GUI (`eframe` / `egui`)**:
  - Click any **Trigger** or **Target** shortcut button and press keys on your keyboard to record a shortcut.
  - Built-in key selector for keys not physically present on compact keyboards (`Media Play/Pause`, `Media Next/Prev Track`, `Volume Up/Down/Mute`, `F13`–`F24`).
  - Live conflict detection if two rules share the same trigger shortcut.
  - One-click **Test** button on every rule to immediately verify output injection.
- **Background Minimize Mode**:
  - Closing the window minimizes it to the taskbar/dock so hotkeys keep running until you click **Quit**.
- **Persistent TOML Configuration**:
  - Automatically saves rules to the OS-standard config path (`%APPDATA%\hotkeys\config.toml` on Windows, `~/Library/Application Support/hotkeys/config.toml` on macOS).

## Platform Setup Notes

### Windows
Uses native Win32 `RegisterHotKey` and `SendInput` APIs via `user32.dll`. No special permissions or drivers are required.

### macOS
Uses Carbon `RegisterEventHotKey` for global shortcut capture and Quartz `CGEventPost` / AppleScript for key and media simulation. On first use of synthetic key injection, grant **Accessibility** permission to the application under **System Settings -> Privacy & Security -> Accessibility**.

## Building & Testing

```bash
# Run keymap, config, and engine unit tests:
cargo test

# Build release binary:
cargo build --release
```

### Downloading Windows & macOS Builds
Pushing to GitHub triggers [`.github/workflows/ci.yml`](.github/workflows/ci.yml), which compiles, lints (`cargo clippy`), tests (`cargo test`), and uploads release binaries natively on `windows-latest` and `macos-latest` runners under the **Actions** tab.
