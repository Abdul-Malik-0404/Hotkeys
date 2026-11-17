# Hotkeys — Cross-Platform Keybind & Media Shortcut Manager

A lightweight Rust desktop and system-tray application for mapping global keyboard shortcuts (`Ctrl`, `Alt`, `Shift`, `Super`/`Win`/`Cmd` + Key) to custom key combinations, media playback keys, or volume controls across **Linux (Wayland & X11)**, **Windows**, and **macOS**.

## Features

- **Interactive Keybind GUI (`eframe` / `egui`)**:
  - Click any **Trigger** or **Target** shortcut button and press keys on your keyboard to record a shortcut.
  - Built-in key selector for keys not physically present on compact keyboards (`Media Play/Pause`, `Media Next/Prev Track`, `Volume Up/Down/Mute`, `F13`–`F24`).
  - Live conflict detection if two rules share the same trigger shortcut.
  - One-click **Test** button on every rule to immediately verify output injection.
- **Background System Tray Support**:
  - Minimizes cleanly to the background / system tray (`StatusNotifierItem` via `ksni` on Linux Wayland/X11) with quick **Enable/Disable** and **Open Settings** controls.
- **Persistent TOML Configuration**:
  - Automatically saves rules to the OS-standard config path (`~/.config/hotkeys/config.toml` on Linux, `%APPDATA%\hotkeys\config.toml` on Windows, `~/Library/Application Support/hotkeys/config.toml` on macOS).

## Platform Setup Notes

### Linux (Wayland & X11)
On Wayland, security isolation prevents X11 APIs from capturing global keys or injecting keystrokes into native Wayland windows. `hotkeys` uses Linux's kernel input subsystem (`evdev` + `/dev/uinput`) so global shortcuts work across **all** Wayland compositors (GNOME, KDE Plasma, Sway, Hyprland, COSMIC) and X11:

```bash
# Allow your user to read keyboard devices and create a virtual uinput keyboard:
sudo usermod -aG input $USER
echo 'KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/99-hotkeys-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```
*(Log out and log back in once after adding your user to the `input` group.)*

### Windows
Uses native Win32 `RegisterHotKey` and `SendInput` APIs via `user32.dll`. No special permissions or drivers are required.

### macOS
Uses Carbon `RegisterEventHotKey` for global shortcut capture and Quartz `CGEventPost` / AppleScript for key and media simulation. On first use of synthetic key injection, grant **Accessibility** permission to the application under **System Settings -> Privacy & Security -> Accessibility**.

## Building & Testing

```bash
# Run cross-platform keymap, config, and engine unit tests:
cargo test

# Build and run the GUI:
cargo run --release
```

### Testing Windows & macOS Builds Without Hardware
Pushing to GitHub triggers [`.github/workflows/ci.yml`](.github/workflows/ci.yml), which compiles, lints (`cargo clippy`), tests (`cargo test`), and packages release binaries natively on `ubuntu-latest`, `windows-latest`, and `macos-latest` runners.
