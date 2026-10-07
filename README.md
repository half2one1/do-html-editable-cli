# do-html-editable-cli ⚡ (v0.1.1)

A high-performance, standalone Rust CLI tool and **Model Context Protocol (MCP)** stdio server that enables AI agents and humans to inspect, edit, word-process, and visually modify static HTML and standalone SVG documents with **zero external server overhead** and **100% air-gapped compliance**.

![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![Version](https://img.shields.io/badge/Version-v0.1.1-brightgreen.svg)
![MCP Supported](https://img.shields.io/badge/MCP-JSON--RPC%202.0-purple.svg)
![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)

---

## 🌟 What's New in v0.1.1

- 🤖 **Model Context Protocol (MCP) Server**: Connect directly to AI hosts (Claude Desktop, Cursor, Antigravity) via `do-html-editable-cli mcp` over stdio with full tool schemas.
- 📝 **CLI Word-Processing Session (`session`)**: Interactive REPL or batch script runner supporting `list`, `get`, `set`, `replace`, `append`, `prepend`, and `save` with optional machine-readable `--json` envelopes.
- 🎯 **One-Shot CLI Editing (`edit`)**: Direct command-line tools to enumerate text nodes, get text, set text, and search-and-replace by index (`#0`, `#1`) or CSS selector.
- 🛡️ **Smart Boundary Backspace & Delete Protection**: Prevents native browser bugs from wiping sentences or merging blocks when Backspacing at offset 0.
- 🎨 **Full SVG Vector Panel Compatibility**: Native support for standalone SVG vector files with fallback append injection.

---

## 📦 Installation & Build

```bash
git clone https://github.com/half2one1/do-html-editable-cli.git
cd do-html-editable-cli
cargo build --release
```

The compiled binary will be located at `target/release/do-html-editable-cli`.

---

## 🤖 MCP Server Setup for AI Agents

`do-html-editable-cli` can act as an MCP stdio server, empowering AI agents to perform surgical word-processing on HTML and SVG files without full-file rewrites.

### Claude Desktop / Cursor / Antigravity Configuration

Add to your MCP config file (e.g. `mcp_config.json` or `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "html-editor": {
      "command": "d:/projects/make-html-editable/target/release/do-html-editable-cli.exe",
      "args": ["mcp"]
    }
  }
}
```

### Exposed MCP Tools

| Tool | Description |
|---|---|
| `html_list_texts` | Enumerate editable text elements with `#idx`, CSS selector, ID, tag, and text preview. |
| `html_get_text` | Retrieve exact text content of an element by index (`#0`) or CSS selector. |
| `html_set_text` | Overwrite text of an element by index or selector, optionally saving to disk. |
| `html_replace_text` | Global or targeted search-and-replace across text nodes. |
| `html_batch_edit` | Atomically apply a list of text edits (set or replace). |
| `html_make_editable` | Inject the air-gapped in-browser visual editing dock into an HTML/SVG document. |

---

## 💻 CLI Usage

### 1. In-Browser Visual Editor (Air-Gapped)

```bash
# Open directly in default browser with floating visual editor:
do-html-editable-cli document.html

# Inject editor without launching browser:
do-html-editable-cli document.html --no-open
```

In-browser controls:
- **Direct Visual Editing**: Click anywhere on text, headings, list items, or tables to edit in-place.
- **[Toggle Edit]** (`Alt+E`): Toggle between edit mode and preview mode.
- **[Clean Export]** (`Ctrl+S` / `Cmd+S`): Deep-clones the revised DOM, purges all injected runtime tags, and downloads `filename.revised.html`.

---

### 2. Interactive & Scriptable Editing Session

Start an interactive word-processing REPL:

```bash
do-html-editable-cli session document.html
```

Or run a non-interactive batch script with machine-readable JSON output:

```bash
do-html-editable-cli session document.html --script edits.txt --json
```

**Session Commands:**
- `list [filter]` — Enumerate all editable text units (`#0`, `#1`, etc.).
- `get <#idx|selector>` — Print current text.
- `set <#idx|selector> <new_text>` — Overwrite element text.
- `append <#idx|selector> <text>` — Append text to element.
- `prepend <#idx|selector> <text>` — Prepend text to element.
- `replace <old> <new>` — Global search and replace.
- `replace in <#idx|selector> <old> <new>` — Targeted search and replace.
- `save [path]` — Save modified document.
- `quit` / `exit` — Exit session.

---

### 3. One-Shot Command-Line Editing

```bash
# List all text elements:
do-html-editable-cli edit document.html list

# List with JSON formatting:
do-html-editable-cli edit document.html list --json

# Get text of element #2:
do-html-editable-cli edit document.html get "#2"

# Set text of element #0 (saved to revised copy or in-place):
do-html-editable-cli edit document.html set "#0" "New Executive Summary" --out revised.html
do-html-editable-cli edit document.html set "#0" "New Executive Summary" --in-place

# Global search and replace:
do-html-editable-cli edit document.html replace "2025" "2026" --in-place
```

---

## 🛠️ Architecture

1. **DOM Processor Engine (`HtmlProcessor`)**: Built on `kuchikiki` (HTML5 parser) with deep DOM tree traversal, CSS selector matching, and XML/HTML serialization.
2. **Deterministic Element Tagging**: Injected visual elements carry `data-editable-engine="true"`, ensuring 100% clean sanitization upon export.
3. **Smart Boundary Protection**: Intercepts `Backspace` and `Delete` at offset 0 of block containers during the capture phase, preventing destructive browser merges and sentence deletions.
4. **Air-Gapped Self-Containment**: Zero external CDN calls, zero local HTTP background daemons.

---

## 📄 License

Dual-licensed under either:
- Apache License, Version 2.0
- MIT License
