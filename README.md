# 🔍 Spotlight

A blazingly fast, cross-platform application launcher built with **Rust** and egui. Press **ALT+D** anywhere to instantly search and launch your applications.

![Rust](https://img.shields.io/badge/rust-%23CE422B.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-blue?style=for-the-badge)

---

## ✨ Features

- 🚀 **Lightning Fast**: Instant app search and launch
- 🌍 **Cross-Platform**: Works on Linux and Windows with platform-specific app discovery
- ⌨️ **Global Hotkey**: Press **ALT+D** anywhere to open/close (even with other apps focused)
- 👻 **Background Daemon**: Runs in the background listening for hotkey events
- 💻 **Native UI**: Beautiful, transparent UI built with egui
- 🎯 **Smart Filtering**: Real-time fuzzy search through your applications
- 📋 **App Metadata**: Shows app names and descriptions
- ⚡ **Zero Configuration**: Just run and go

---

## 🖥️ Platforms

### Linux
- Scans `.desktop` files from FreeDesktop directories
- Supports Flatpak and Snap applications
- Works with user-local applications

### Windows
- Reads from Windows Registry (`HKEY_LOCAL_MACHINE\Software\...\Uninstall`)
- Detects both 32-bit and 64-bit applications
- Integrates with installed applications

---

## 🚀 Installation

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))

### Build from Source

```bash
git clone https://github.com/yourusername/spotlight
cd spotlight
cargo build --release
```

The compiled binary will be at `target/release/spotlight`

#### Linux
```bash
# Run directly
./target/release/spotlight

# Or install to system
cargo install --path .
```

#### Windows
```bash
# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Or install
cargo install --path .
```

---

## 🎮 Usage

### Quick Start

```bash
cargo run --release
```

The app will start in the background and listen for hotkeys.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| <kbd>ALT</kbd> + <kbd>D</kbd> | Show/Hide Spotlight |
| <kbd>↓</kbd> / <kbd>↑</kbd> | Navigate results |
| <kbd>Enter</kbd> | Launch selected app |
| <kbd>ESC</kbd> | Hide Spotlight |

### Example Workflow

```
1. Press ALT+D anywhere
2. Type "chrome"
3. Press Enter to launch
4. Spotlight hides in background
```

---

## 🏗️ Project Structure

```
spotlight/
├── Cargo.toml              # Dependencies & metadata
└── src/
    └── main.rs            # Complete app implementation
        ├── UI rendering (egui)
        ├── Hotkey listener (rdev)
        ├── App discovery
        │   ├── Linux: scan_linux_apps()
        │   └── Windows: scan_windows_apps()
        └── Launch handler
```

---

## 💡 What I Learned (Rust Journey)

This project covers essential Rust concepts:

- **Concurrency**: Multi-threaded hotkey listening with `std::thread`
- **Channel Communication**: `std::sync::mpsc` for inter-thread messaging
- **Conditional Compilation**: Platform-specific code with `#[cfg(...)]`
- **Error Handling**: Result types and error propagation
- **GUI Development**: egui framework
- **Process Management**: `std::process::Command`
- **File I/O**: Reading and parsing desktop files
- **Registry Access**: Windows registry interaction with `winreg`
- **System Integration**: Global event listening with `rdev`

---

## 📦 Dependencies

| Crate | Purpose |
|-------|---------|
| `eframe` | GUI framework (egui wrapper) |
| `egui` | Immediate-mode UI rendering |
| `rdev` | Global keyboard event listener |
| `winreg` | Windows Registry access (Windows only) |
| `parking_lot` | Optimized synchronization primitives |

---

## 🔧 Technical Highlights

### Cross-Platform App Discovery
```rust
#[cfg(target_os = "linux")]
fn scan_linux_apps() -> Vec<App> { /* ... */ }

#[cfg(target_os = "windows")]
fn scan_windows_apps() -> Vec<App> { /* ... */ }
```

### Global Hotkey Listening
Background thread with event listener:
```rust
std::thread::spawn(|| {
    rdev::listen(|event| {
        // Handle ALT+D globally
    })
})
```

### Background Daemon Mode
- App starts invisible
- Hotkey toggles visibility
- Accepts input only when visible
- Runs continuously until closed

---

## 🎨 UI Screenshot

```
┌──────────────── Spotlight ────────────────┐
│ 🔍 Search applications...                 │
├───────────────────────────────────────────┤
│ • Firefox                                 │
│   A fast web browser                      │
│ • Visual Studio Code                 (sel)│
│   Code editing redefined                  │
│ • Spotify                                 │
│   Music streaming                         │
│ • Discord                                 │
│   Chat & voice                            │
│                                           │
└───────────────────────────────────────────┘
```

---

## 🚀 Performance

- **App Scanning**: ~50-200ms (cached on startup)
- **Search Filtering**: Real-time as you type
- **Launch**: < 100ms

---

## 🐛 Known Limitations

- **Linux**: Only scans FreeDesktop standard locations
- **Windows**: Requires to be running as admin for full registry access
- **Hotkey**: Platform-specific event handling may vary by desktop environment

---

## 🔮 Future Ideas

- [ ] Configuration file (custom hotkey, theme)
- [ ] Recently used apps
- [ ] App categories/groups
- [ ] Custom commands/scripts
- [ ] Plugin system
- [ ] macOS support
- [ ] Web shortcuts
- [ ] System commands integration

---

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## 🙏 Acknowledgments

- [egui](https://github.com/emilk/egui) - Immediate-mode GUI
- [rdev](https://github.com/enigo-rs/rdev) - Global input event listener
- Inspired by macOS Spotlight & Linux rofi/albert

---

## 📚 Learning Resources Used

- [The Rust Book](https://doc.rust-lang.org/book/)
- [egui Documentation](https://docs.rs/egui/)
- [rdev Documentation](https://docs.rs/rdev/)
- Community feedback and Rust forums

---

**Made with ❤️ to learn Rust** 🦀

_Feel free to fork, star ⭐, and contribute!_
