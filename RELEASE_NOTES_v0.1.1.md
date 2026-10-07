# Release Notes - v0.1.1

**Release Date:** October 2026  
**Repository:** [half2one1/do-html-editable-cli](https://github.com/half2one1/do-html-editable-cli)  
**Binary Name:** `do-html-editable-cli`

---

## 🌟 Overview

`do-html-editable-cli` v0.1.1 is a major upgrade transforming the tool from a browser-only injector into a full **word-processing engine** and **AI agent automation server**. It introduces native **Model Context Protocol (MCP)** server capabilities and interactive/scriptable **CLI editing sessions**, allowing AI agents and CLI users to surgical-edit text in static HTML and SVG documents with sub-second precision.

---

## 🚀 Key Features in v0.1.1

### 1. Model Context Protocol (MCP) stdio Server (`mcp`)
- Standards-compliant JSON-RPC 2.0 protocol over `stdin`/`stdout`.
- Connects directly to AI hosts including Claude Desktop, Cursor, and Antigravity IDE.
- Exposes 6 specialized tools:
  - `html_list_texts`: Structured enumeration with index `#0`, `#1`, CSS selectors, IDs, and text previews.
  - `html_get_text`: Accurate reading of specific element texts by index or selector.
  - `html_set_text`: Targeted element text replacement with optional in-place writing.
  - `html_replace_text`: Document-wide or scoped search-and-replace.
  - `html_batch_edit`: Atomic execution of multi-step edit batches.
  - `html_make_editable`: Automated browser visual editor injection.

### 2. CLI-Based Word-Processing Session (`session`)
- Interactive REPL terminal mode (`do-html-editable-cli session file.html`).
- Batch script execution mode (`--script script.txt`).
- Machine-readable JSON envelope support (`--json`).
- Supported commands: `list`, `get`, `set`, `append`, `prepend`, `replace`, `replace in`, `status`, `save`, and `quit`.

### 3. One-Shot Command-Line Editing (`edit`)
- Direct subcommands for shell scripting and automation:
  - `edit <FILE> list [--json]`
  - `edit <FILE> get <TARGET>`
  - `edit <FILE> set <TARGET> <TEXT> [--in-place] [--out <PATH>]`
  - `edit <FILE> replace <OLD> <NEW> [--target <TARGET>] [--in-place] [--out <PATH>]`

### 4. DOM Parsing & Word-Processing Engine (`HtmlProcessor`)
- Powered by `kuchikiki` and `html5ever`.
- Handles both standard HTML documents and standalone SVG vector panels.
- Deep text node tree traversal guaranteeing preservation of surrounding HTML/SVG tags, attributes, and styles.

### 5. Smart Boundary Backspace & Delete Protection
- Fixed the native browser quirk where pressing `Backspace` at offset 0 after typing text destroys the entire sentence or list item.
- Preserves empty-line deletion while guarding non-empty block content.

---

## 📦 Usage Examples

```bash
# 1. Start MCP Server for AI Agents
do-html-editable-cli mcp

# 2. Run Interactive Word-Processing Session
do-html-editable-cli session document.html

# 3. List and Edit via One-Shot CLI
do-html-editable-cli edit document.html list
do-html-editable-cli edit document.html set "#0" "New Document Title" --in-place

# 4. In-Browser Visual Editor (Original Workflow)
do-html-editable-cli document.html
```
