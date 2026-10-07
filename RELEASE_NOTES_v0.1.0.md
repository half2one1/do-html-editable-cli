# Release Notes - v0.1.0

**Release Date:** October 2026  
**Repository:** [half2one1/do-html-editable-cli](https://github.com/half2one1/do-html-editable-cli)  
**Binary Name:** `do-html-editable-cli`

---

## 🌟 Overview

`do-html-editable-cli` v0.1.0 is the foundational release of a lightweight, single-file CLI application designed to transform static, AI-generated HTML files (and standalone SVG vector files) into visually editable browser documents without hosting any local microserver.

---

## 🚀 Key Features in v0.1.0

### 1. 100% Air-Gapped & Zero Server Overhead
- Operates natively under the browser's `file://` protocol.
- No local HTTP servers (Axum, Actix, Python HTTP) spawned or required.
- Self-contained payload: all CSS styling and JavaScript runtimes are embedded in the compiled binary with zero external CDN or framework dependencies.

### 2. Non-Destructive In-Browser Editing
- Reads `filename.html` and outputs `filename.editable.html` in the same directory, guaranteeing original files are never overwritten.
- Enables browser-native `document.designMode = "on"` for seamless in-place editing of text, headings, lists, tables, and layouts.

### 3. Draggable Floating Control Dock
- Minimalist dark glassmorphism toolbar docked with high `z-index`.
- Real-time visual status indicator (🟢 **Editing** vs ⚪ **View Mode**).
- **[Toggle Edit]** (Shortcut: `Alt+E`): Toggles between interactive editing and browsing mode.
- Repositionable: Click and drag anywhere on the toolbar background.

### 4. Clean Export Engine (`make-.editable.html-non-editable`)
- One-click **[Clean Export]** (Shortcut: `Ctrl+S` / `Cmd+S`).
- Deep-clones the revised DOM and purges every injected `<style>`, `<div>`, and `<script>` element tagged with `data-editable-engine="true"`.
- Reconstructs original `<!DOCTYPE>` declarations.
- Initiates a native client browser `Blob` download of `filename.revised.html` with zero editor code footprint.

### 5. Smart Boundary Backspace & Delete Protection
- Intercepts keystrokes during the capture phase when the caret is collapsed at offset `0` of block elements (`<p>`, `<li>`, `<h1>`-`<h6>`, `<div>`, etc.).
- Eliminates the infamous browser bug where pressing `Backspace` at the beginning of a line or after deleting newly added strings destroys the entire sentence or merges blocks.
- Symmetrically protects forward `Delete` at block endings.

### 6. Full SVG Vector Panel Compatibility
- Gracefully handles standalone SVG files (such as A2 exhibition panels) lacking `</body>` tags by falling back to boundary append injection.
- Preserves all SVG coordinates, gradients, `<text>`, and `<tspan>` nodes.

---

## 📦 Getting Started

```bash
# Clone and build
git clone https://github.com/half2one1/do-html-editable-cli.git
cd do-html-editable-cli
cargo build --release

# Run on any HTML file
./target/release/do-html-editable-cli path/to/document.html
```
