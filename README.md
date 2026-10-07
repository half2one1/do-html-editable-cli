# do-html-editable-cli ⚡

A lightweight, standalone, single-file Rust CLI application that transforms static, AI-generated HTML files into visually editable browser documents with **zero external server overhead** and **100% air-gapped compliance**.

![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)
![Dependencies](https://img.shields.io/badge/Dependencies-Minimal-green.svg)

---

## 🚀 Key Highlights

- **Air-Gapped Compliance**: Zero local background microservers (no Actix, Axum, or Node.js). Everything runs directly via the browser's native `file://` protocol.
- **Zero Internet Dependencies**: Injected styles and JavaScript runtime are 100% self-contained inside the compiled Rust binary. No CDNs, no external CSS/JS frameworks.
- **Non-Destructive Processing**: Reads `filename.html` and creates `filename.editable.html` in the same directory, leaving the original source file completely untouched.
- **Clean Export Engine**: One-click export purges every trace of injected runtime code, styles, and toolbar elements, generating a pure, sanitized `filename.revised.html`.
- **Smart Boundary Backspace & Delete Protection**: Prevents native browser bugs where pressing `Backspace` at offset 0 collapses adjacent blocks or deletes entire list items and sentences.
- **Modern Floating Dock**: Draggable, minimalist dark glassmorphism toolbar with real-time status indication, keyboard shortcuts (`Ctrl+S` / `Cmd+S`, `Alt+E`), and click-to-edit toggle.

---

## 📦 Installation & Build

Ensure you have Rust and Cargo installed:

```bash
git clone https://github.com/half2one1/do-html-editable-cli.git
cd do-html-editable-cli
cargo build --release
```

The optimized standalone binary will be located at `target/release/do-html-editable-cli`.

---

## 💻 Usage

```bash
# Transform an HTML file and open it immediately in your default browser:
do-html-editable-cli path/to/document.html

# Process an HTML file without automatically opening the browser:
do-html-editable-cli path/to/document.html --no-open
```

### In-Browser Controls

Once the `.editable.html` file opens in your browser:
- **Direct Visual Editing**: Click anywhere on text, headings, list items, or tables to edit them directly.
- **[Toggle Edit]** (Shortcut: `Alt+E`): Switch between interactive Edit Mode and Preview Mode.
- **[Clean Export]** (Shortcut: `Ctrl+S` / `Cmd+S`): Deep-clones the revised DOM, sanitizes all injected editor components, and triggers a clean native client download of `filename.revised.html`.
- **Repositionable Dock**: Drag the floating toolbar by its background to place it anywhere on screen.

---

## 🛠️ Architecture

1. **Case-Insensitive Byte Scanner**: Locates the closing `</body>` tag (or falls back to appending at document end) using zero-allocation byte slice comparison.
2. **Deterministic Element Tagging**: All injected elements (`<style>`, `<div>`, `<script>`) carry `data-editable-engine="true"`.
3. **DOM Sanitization**: Deep-clones the root DOM, strips all tagged nodes, reconstructs the original `<!DOCTYPE>` declaration, and generates a native browser `Blob` download.

---

## 📄 License

Dual-licensed under either of:
- Apache License, Version 2.0
- MIT License
