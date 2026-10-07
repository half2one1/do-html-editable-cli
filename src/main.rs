//! # do-html-editable-cli (v0.1.1)
//!
//! A lightweight, standalone CLI tool and MCP server that provides word-processing,
//! AI-agent text editing, and air-gapped in-browser visual editing for static HTML
//! and SVG documents with zero external server overhead.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub mod mcp;
pub mod processor;
pub mod session;

use processor::HtmlProcessor;
use session::EditingSession;

/// Command-line argument configuration
#[derive(Parser, Debug)]
#[command(
    name = "do-html-editable-cli",
    author = "Antigravity Engineering",
    version = "0.1.1",
    about = "Word-processing and in-browser visual editing CLI & MCP server for static HTML and SVG files.",
    long_about = "A high-performance air-gapped tool for AI agents and humans to inspect, edit, and visually modify\n\
                  static HTML and SVG documents with subcommands for MCP, interactive editing sessions, and browser injection."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Direct target HTML/SVG file to make editable in browser (default workflow)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Do not automatically open the editable file in the default browser
    #[arg(long, default_value_t = false)]
    no_open: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the air-gapped in-browser visual editor
    Make {
        /// Target HTML or SVG file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Skip launching the default browser
        #[arg(long, default_value_t = false)]
        no_open: bool,
    },

    /// Start an interactive or scriptable CLI editing session
    Session {
        /// Target HTML or SVG file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Batch script file to execute non-interactively
        #[arg(long, value_name = "SCRIPT")]
        script: Option<PathBuf>,

        /// Output results as machine-readable JSON envelopes
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Save modifications directly in-place to the source file
        #[arg(long, default_value_t = false)]
        in_place: bool,
    },

    /// Perform one-shot command-line text inspection or editing
    Edit {
        /// Target HTML or SVG file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        #[command(subcommand)]
        action: EditActions,
    },

    /// Run as a Model Context Protocol (MCP) stdio server for AI agents
    Mcp,
}

#[derive(Subcommand, Debug)]
enum EditActions {
    /// List all editable text units in the file
    List {
        /// Optional substring filter
        #[arg(long)]
        filter: Option<String>,

        /// Maximum number of items
        #[arg(long)]
        limit: Option<usize>,

        /// Output in JSON format
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Get text of a specific element by #index or CSS selector
    Get {
        /// Target #index (#0) or CSS selector
        target: String,
    },

    /// Set text of a specific element
    Set {
        /// Target #index (#0) or CSS selector
        target: String,

        /// New text string
        text: String,

        /// Modify source file in-place
        #[arg(long, default_value_t = false)]
        in_place: bool,

        /// Optional output destination file
        #[arg(long)]
        out: Option<PathBuf>,
    },

    /// Search and replace text occurrences
    Replace {
        /// String to find
        old: String,

        /// Replacement string
        new: String,

        /// Target element scope (#index or CSS selector)
        #[arg(long)]
        target: Option<String>,

        /// Modify source file in-place
        #[arg(long, default_value_t = false)]
        in_place: bool,

        /// Optional output destination file
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

/// Injected styling, UI toolbar markup, and runtime script.
/// Uses Rust raw string literals (r###""###) for clean separation and zero external dependencies.
const INJECTED_ENGINE_TEMPLATE: &str = r###"
<style id="__mhe_engine_style" data-editable-engine="true">
  #__mhe_toolbar {
    all: initial;
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 2147483647;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 13px;
    line-height: 1.4;
    color: #f4f4f5;
    background: rgba(18, 18, 24, 0.92);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    padding: 6px 8px 6px 14px;
    box-shadow: 0 14px 34px -4px rgba(0, 0, 0, 0.55), 0 0 0 1px rgba(255, 255, 255, 0.08);
    display: flex;
    align-items: center;
    gap: 10px;
    user-select: none;
    -webkit-user-select: none;
    cursor: grab;
    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s ease;
  }
  #__mhe_toolbar:active {
    cursor: grabbing;
  }
  #__mhe_toolbar:hover {
    box-shadow: 0 18px 42px -4px rgba(0, 0, 0, 0.65), 0 0 0 1px rgba(255, 255, 255, 0.16);
  }
  .__mhe_status_group {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding-right: 2px;
    pointer-events: none;
  }
  .__mhe_indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: #22c55e;
    box-shadow: 0 0 10px rgba(34, 197, 94, 0.85);
    transition: background-color 0.2s ease, box-shadow 0.2s ease;
  }
  .__mhe_indicator.is-disabled {
    background-color: #71717a;
    box-shadow: none;
  }
  .__mhe_status_text {
    font-size: 12px;
    font-weight: 500;
    color: #d4d4d8;
    letter-spacing: 0.01em;
  }
  .__mhe_divider {
    width: 1px;
    height: 18px;
    background: rgba(255, 255, 255, 0.14);
    pointer-events: none;
  }
  .__mhe_btn {
    all: initial;
    box-sizing: border-box;
    cursor: pointer;
    border-radius: 9999px;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    padding: 6px 14px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    white-space: nowrap;
    transition: all 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .__mhe_btn_toggle {
    background: rgba(255, 255, 255, 0.08);
    color: #f4f4f5;
    border: 1px solid rgba(255, 255, 255, 0.12);
  }
  .__mhe_btn_toggle:hover {
    background: rgba(255, 255, 255, 0.16);
    border-color: rgba(255, 255, 255, 0.22);
    color: #ffffff;
  }
  .__mhe_btn_toggle:active {
    transform: scale(0.96);
  }
  .__mhe_btn_export {
    background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
    color: #ffffff;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 10px rgba(37, 99, 235, 0.45);
  }
  .__mhe_btn_export:hover {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    box-shadow: 0 4px 14px rgba(37, 99, 235, 0.65);
  }
  .__mhe_btn_export:active {
    transform: scale(0.96);
  }
  .__mhe_btn_export.is-success {
    background: linear-gradient(135deg, #059669 0%, #047857 100%);
    box-shadow: 0 2px 10px rgba(5, 150, 105, 0.55);
  }
</style>

<div id="__mhe_toolbar" data-editable-engine="true" role="toolbar" aria-label="Visual Editor Toolbar" title="Drag to reposition">
  <div class="__mhe_status_group">
    <span id="__mhe_indicator" class="__mhe_indicator"></span>
    <span id="__mhe_status_text" class="__mhe_status_text">Editing</span>
  </div>
  <div class="__mhe_divider"></div>
  <button id="__mhe_btn_toggle" class="__mhe_btn __mhe_btn_toggle" type="button" title="Toggle browser editing mode (Shortcut: Alt+E)">Toggle Edit</button>
  <button id="__mhe_btn_export" class="__mhe_btn __mhe_btn_export" type="button" title="Clean export sanitized HTML without editor code (Shortcut: Ctrl+S)">Clean Export</button>
</div>

<script id="__mhe_engine_script" data-editable-engine="true">
(function() {
  "use strict";

  // Target filename for clean export download
  const EXPORT_FILENAME = "__DEFAULT_EXPORT_FILENAME__";

  // Initialize document in visual editing mode
  document.designMode = "on";
  let isEditing = true;

  const toolbar = document.getElementById("__mhe_toolbar");
  const toggleBtn = document.getElementById("__mhe_btn_toggle");
  const exportBtn = document.getElementById("__mhe_btn_export");
  const indicator = document.getElementById("__mhe_indicator");
  const statusText = document.getElementById("__mhe_status_text");

  function updateToolbarState() {
    if (isEditing) {
      indicator.classList.remove("is-disabled");
      statusText.textContent = "Editing";
      toggleBtn.textContent = "Disable Edit";
    } else {
      indicator.classList.add("is-disabled");
      statusText.textContent = "View Mode";
      toggleBtn.textContent = "Enable Edit";
    }
  }

  // Initial button label sync
  updateToolbarState();

  // Handle [Toggle Edit]
  toggleBtn.addEventListener("click", function(event) {
    event.preventDefault();
    event.stopPropagation();
    isEditing = !isEditing;
    document.designMode = isEditing ? "on" : "off";
    updateToolbarState();
  });

  // Draggable toolbar logic
  let isDragging = false;
  let dragStartX, dragStartY, initialLeft, initialTop;

  toolbar.addEventListener("mousedown", function(e) {
    if (e.target.tagName === "BUTTON") return;
    isDragging = true;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    const rect = toolbar.getBoundingClientRect();
    toolbar.style.bottom = "auto";
    toolbar.style.right = "auto";
    toolbar.style.left = rect.left + "px";
    toolbar.style.top = rect.top + "px";
    initialLeft = rect.left;
    initialTop = rect.top;
  });

  window.addEventListener("mousemove", function(e) {
    if (!isDragging) return;
    const dx = e.clientX - dragStartX;
    const dy = e.clientY - dragStartY;
    const maxX = window.innerWidth - toolbar.offsetWidth - 8;
    const maxY = window.innerHeight - toolbar.offsetHeight - 8;
    toolbar.style.left = Math.max(8, Math.min(maxX, initialLeft + dx)) + "px";
    toolbar.style.top = Math.max(8, Math.min(maxY, initialTop + dy)) + "px";
  });

  window.addEventListener("mouseup", function() {
    isDragging = false;
  });

  // Boundary Protection: Prevents browser from destroying block structures,
  // merging distinct paragraphs/headings/list-items, or wiping out sentences
  // when Backspace is pressed at the start of a line or element.
  window.addEventListener("keydown", function(event) {
    if (!isEditing) return;

    // Keyboard shortcuts (Ctrl+S / Cmd+S for export, Alt+E for toggle)
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      exportBtn.click();
      return;
    } else if (event.altKey && event.key.toLowerCase() === "e") {
      event.preventDefault();
      toggleBtn.click();
      return;
    }

    if (event.key !== "Backspace" && event.key !== "Delete") return;

    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return;
    const range = sel.getRangeAt(0);

    // Only protect collapsed carets (when not deleting an explicit highlighted text selection)
    if (!range.collapsed) return;

    // Helper: Find closest block-level container or structural item
    function findBoundaryContainer(node) {
      const BLOCK_TAGS = new Set([
        "P", "H1", "H2", "H3", "H4", "H5", "H6", "LI", "DIV",
        "BLOCKQUOTE", "PRE", "TD", "TH", "HEADER", "FOOTER",
        "SECTION", "ARTICLE", "ASIDE", "FIGCAPTION", "FIGURE",
        "DT", "DD", "TEXT", "TSPAN"
      ]);
      let curr = (node.nodeType === Node.TEXT_NODE) ? node.parentNode : node;
      while (curr && curr !== document.body && curr !== document.documentElement) {
        if (curr.hasAttribute && curr.hasAttribute("data-editable-engine")) {
          return null;
        }
        if (BLOCK_TAGS.has(curr.tagName)) {
          return curr;
        }
        try {
          const display = window.getComputedStyle(curr).display;
          if (display === "block" || display === "flex" || display === "grid" || display === "list-item" || display === "table-cell") {
            return curr;
          }
        } catch (e) {}
        curr = curr.parentNode;
      }
      return curr;
    }

    const container = findBoundaryContainer(range.startContainer);
    if (!container) return;

    if (event.key === "Backspace") {
      try {
        const preRange = document.createRange();
        preRange.selectNodeContents(container);
        preRange.setEnd(range.startContainer, range.startOffset);
        const textBefore = preRange.toString();

        // If caret is at the start of the block and the block contains non-empty text,
        // prevent the destructive merge or deletion of the sentence!
        if (textBefore.length === 0 && container.textContent.trim().length > 0) {
          event.preventDefault();
          event.stopPropagation();
        }
      } catch (err) {}
    } else if (event.key === "Delete") {
      try {
        const postRange = document.createRange();
        postRange.setStart(range.startContainer, range.startOffset);
        postRange.setEnd(container, container.childNodes.length);
        const textAfter = postRange.toString();

        // If caret is at the end of the block, prevent pulling and destroying next block
        if (textAfter.length === 0 && container.textContent.trim().length > 0) {
          event.preventDefault();
          event.stopPropagation();
        }
      } catch (err) {}
    }
  }, true);

  // Handle [Clean Export] - Sanitize and download clean HTML
  exportBtn.addEventListener("click", function(event) {
    event.preventDefault();
    event.stopPropagation();

    const previousEditingState = isEditing;

    // Step 1: Turn off document.designMode
    document.designMode = "off";

    // Clear active text selections so they don't persist into the DOM snapshot
    if (window.getSelection) {
      window.getSelection().removeAllRanges();
    }

    try {
      // Step 2: Clone the current document structure
      const domClone = document.documentElement.cloneNode(true);

      // Step 3: Completely remove all injected tags (style, toolbar div, script)
      const engineElements = domClone.querySelectorAll('[data-editable-engine="true"]');
      engineElements.forEach(function(el) {
        if (el && el.parentNode) {
          el.parentNode.removeChild(el);
        }
      });

      // Defensive verification: ensure toolbar ID does not exist in clone
      const residualToolbar = domClone.querySelector("#__mhe_toolbar");
      if (residualToolbar && residualToolbar.parentNode) {
        residualToolbar.parentNode.removeChild(residualToolbar);
      }

      // Check if this document originated as an SVG file
      const isSvgExport = EXPORT_FILENAME.endsWith(".svg");
      let cleanContent = "";

      if (isSvgExport) {
        const svgEl = domClone.querySelector("svg");
        if (svgEl) {
          cleanContent = '<?xml version="1.0" encoding="UTF-8"?>\n' + svgEl.outerHTML;
        } else {
          cleanContent = domClone.outerHTML;
        }
      } else {
        // Step 4: Reconstruct doctype declaration
        let doctypeString = "<!DOCTYPE html>\n";
        if (document.doctype) {
          doctypeString = "<!DOCTYPE " + document.doctype.name;
          if (document.doctype.publicId) {
            doctypeString += ' PUBLIC "' + document.doctype.publicId + '"';
          }
          if (document.doctype.systemId) {
            doctypeString += ' "' + document.doctype.systemId + '"';
          }
          doctypeString += ">\n";
        }
        cleanContent = doctypeString + domClone.outerHTML;
      }

      // Step 5: Native browser Blob download trigger
      const mimeType = isSvgExport ? "image/svg+xml;charset=utf-8" : "text/html;charset=utf-8";
      const blob = new Blob([cleanContent], { type: mimeType });
      const downloadUrl = URL.createObjectURL(blob);
      const downloadLink = document.createElement("a");
      downloadLink.href = downloadUrl;
      downloadLink.download = EXPORT_FILENAME;
      document.body.appendChild(downloadLink);
      downloadLink.click();
      document.body.removeChild(downloadLink);

      setTimeout(function() {
        URL.revokeObjectURL(downloadUrl);
      }, 1500);

      // UI confirmation feedback
      const originalLabel = exportBtn.textContent;
      exportBtn.textContent = "Exported ✓";
      exportBtn.classList.add("is-success");
      setTimeout(function() {
        exportBtn.textContent = originalLabel;
        exportBtn.classList.remove("is-success");
      }, 2000);

    } catch (error) {
      console.error("Clean Export Error:", error);
      alert("Export failed: " + error.message);
    } finally {
      // Restore previous editing state for uninterrupted workflow
      if (previousEditingState) {
        document.designMode = "on";
      }
    }
  });
})();
</script>
"###;

/// Generates the injected engine block customized with the target export filename.
pub fn generate_injected_engine(export_filename: &str) -> String {
    INJECTED_ENGINE_TEMPLATE.replace("__DEFAULT_EXPORT_FILENAME__", export_filename)
}

/// Finds the byte offset of the closing `</body>` tag (case-insensitive) in the HTML content.
/// Returns `None` if no `</body>` tag is present.
pub fn find_closing_body_offset(html: &str) -> Option<usize> {
    let target = b"</body>";
    let bytes = html.as_bytes();
    if bytes.len() < target.len() {
        return None;
    }

    // Search backwards from the end of the document to find the final closing tag
    (0..=(bytes.len() - target.len())).rev().find(|&i| {
        bytes[i..i + target.len()].eq_ignore_ascii_case(target)
    })
}

/// Injects the self-contained editor engine into the HTML string.
/// Injects right before the closing `</body>` tag if present; otherwise appends to the end.
pub fn inject_editor_runtime(html: &str, export_filename: &str) -> String {
    let engine_block = generate_injected_engine(export_filename);

    if let Some(offset) = find_closing_body_offset(html) {
        let mut output = String::with_capacity(html.len() + engine_block.len());
        output.push_str(&html[..offset]);
        output.push_str(&engine_block);
        output.push_str(&html[offset..]);
        output
    } else {
        let mut output = String::with_capacity(html.len() + engine_block.len() + 1);
        output.push_str(html);
        output.push('\n');
        output.push_str(&engine_block);
        output
    }
}

/// Computes the output `.editable.html` path and clean export `.revised.html` / `.revised.svg` name.
/// Ensures the original file is preserved in the same directory without overwriting.
pub fn derive_file_paths(input_path: &Path) -> (PathBuf, String) {
    let parent_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    let raw_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    // Clean stem if the user happens to pass an already .editable file
    let clean_stem = if let Some(stripped) = raw_stem.strip_suffix(".editable") {
        stripped
    } else {
        raw_stem
    };

    let is_svg = input_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("svg"))
        .unwrap_or(false);

    let editable_filename = format!("{}.editable.html", clean_stem);
    let revised_filename = if is_svg {
        format!("{}.revised.svg", clean_stem)
    } else {
        format!("{}.revised.html", clean_stem)
    };
    let editable_path = parent_dir.join(editable_filename);

    (editable_path, revised_filename)
}

/// Handles the `make` action (browser-based visual editor injection)
fn handle_make(file_path: &Path, no_open: bool) {
    if !file_path.exists() {
        eprintln!("Error: Target file '{}' does not exist.", file_path.display());
        process::exit(1);
    }

    if !file_path.is_file() {
        eprintln!("Error: Path '{}' is a directory, not a file.", file_path.display());
        process::exit(1);
    }

    let raw_html = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: Failed to read file '{}': {}", file_path.display(), err);
            process::exit(1);
        }
    };

    let (editable_path, export_filename) = derive_file_paths(file_path);
    let modified_html = inject_editor_runtime(&raw_html, &export_filename);

    if let Err(err) = fs::write(&editable_path, &modified_html) {
        eprintln!(
            "Error: Failed to write editable file '{}': {}",
            editable_path.display(),
            err
        );
        process::exit(1);
    }

    println!("============================================================");
    println!(" do-html-editable-cli - Visual Browser Editor");
    println!("============================================================");
    println!(" [✓] Source File   : {}", file_path.display());
    println!(" [✓] Editable File : {}", editable_path.display());
    println!(" [✓] Clean Export  : {}", export_filename);
    println!("------------------------------------------------------------");
    println!(" Features Active:");
    println!("  • In-browser visual editing via native document.designMode");
    println!("  • Floating toolbar with [Toggle Edit] and [Clean Export]");
    println!("  • Boundary Backspace & Delete protection active");
    println!("  • 100% offline self-contained (zero CDNs, zero background server)");
    println!("============================================================");

    if !no_open {
        println!("Spawning native default browser for '{}'...", editable_path.display());
        match opener::open(&editable_path) {
            Ok(_) => println!("Browser launched successfully."),
            Err(err) => {
                eprintln!(
                    "Notice: Could not automatically launch system browser: {}.\n\
                     You can open the file manually: {}",
                    err,
                    editable_path.display()
                );
            }
        }
    } else {
        println!("Note: Auto-open skipped (--no-open specified).");
    }
}

/// Handles the `session` command
fn handle_session(file_path: &Path, script: Option<&Path>, json_mode: bool, in_place: bool) {
    let mut session = match EditingSession::new(file_path, in_place, json_mode) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error initializing session: {}", err);
            process::exit(1);
        }
    };

    if let Some(script_file) = script {
        if let Err(err) = session.run_script(script_file) {
            eprintln!("Script execution failed: {}", err);
            process::exit(1);
        }
    } else {
        if let Err(err) = session.run_interactive() {
            eprintln!("Interactive session error: {}", err);
            process::exit(1);
        }
    }
}

/// Handles the one-shot `edit` command
fn handle_edit(file_path: &Path, action: EditActions) {
    let proc = match HtmlProcessor::from_file(file_path) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            process::exit(1);
        }
    };

    match action {
        EditActions::List { filter, limit, json } => {
            let texts = proc.list_texts(filter.as_deref(), limit);
            if json {
                let out = serde_json::to_string_pretty(&texts).unwrap();
                println!("{}", out);
            } else {
                println!("Editable texts in '{}' (Total: {}):", file_path.display(), texts.len());
                for t in &texts {
                    let id_info = t.id.as_deref().map(|id| format!(" #{}", id)).unwrap_or_default();
                    println!("[#{:<3}] <{}{}>: {}", t.index, t.tag, id_info, t.text);
                }
            }
        }

        EditActions::Get { target } => match proc.get_text(&target) {
            Ok(text) => println!("{}", text),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },

        EditActions::Set {
            target,
            text,
            in_place,
            out,
        } => {
            if let Err(e) = proc.set_text(&target, &text) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            let dest = if let Some(out_path) = out {
                Some(out_path)
            } else if in_place {
                Some(file_path.to_path_buf())
            } else {
                // Default to .revised.html / .revised.svg
                let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
                let ext = if proc.is_svg { "svg" } else { "html" };
                let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
                Some(parent.join(format!("{}.revised.{}", stem, ext)))
            };

            match proc.save(dest.as_ref()) {
                Ok(saved) => println!("Updated '{}' -> saved to '{}'.", target, saved.display()),
                Err(e) => {
                    eprintln!("Error saving document: {}", e);
                    process::exit(1);
                }
            }
        }

        EditActions::Replace {
            old,
            new,
            target,
            in_place,
            out,
        } => {
            match proc.replace_text(&old, &new, target.as_deref()) {
                Ok(count) => {
                    let dest = if let Some(out_path) = out {
                        Some(out_path)
                    } else if in_place {
                        Some(file_path.to_path_buf())
                    } else {
                        let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
                        let ext = if proc.is_svg { "svg" } else { "html" };
                        let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
                        Some(parent.join(format!("{}.revised.{}", stem, ext)))
                    };

                    match proc.save(dest.as_ref()) {
                        Ok(saved) => println!(
                            "Replaced {} occurrence(s) of '{}' with '{}' -> saved to '{}'.",
                            count,
                            old,
                            new,
                            saved.display()
                        ),
                        Err(e) => {
                            eprintln!("Error saving document: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Make { file, no_open } => handle_make(&file, no_open),
            Commands::Session {
                file,
                script,
                json,
                in_place,
            } => handle_session(&file, script.as_deref(), json, in_place),
            Commands::Edit { file, action } => handle_edit(&file, action),
            Commands::Mcp => {
                if let Err(e) = mcp::run_mcp_server() {
                    eprintln!("MCP Server error: {}", e);
                    process::exit(1);
                }
            }
        }
    } else if let Some(file_path) = cli.file {
        // Direct argument: default workflow (make editable in browser)
        handle_make(&file_path, cli.no_open);
    } else {
        eprintln!("Error: No HTML/SVG file or command specified.\nRun 'do-html-editable-cli --help' for usage.");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_closing_body_tag() {
        let html_lower = "<html><head></head><body><h1>Hello</h1></body></html>";
        let offset = find_closing_body_offset(html_lower).expect("Should find </body>");
        assert_eq!(&html_lower[offset..offset + 7], "</body>");

        let html_upper = "<html><head></head><BODY><H1>Hello</H1></BODY></html>";
        let offset_upper = find_closing_body_offset(html_upper).expect("Should find </BODY>");
        assert_eq!(&html_upper[offset_upper..offset_upper + 7], "</BODY>");

        let html_no_body = "<div>Just a fragment</div>";
        assert!(find_closing_body_offset(html_no_body).is_none());
    }

    #[test]
    fn test_inject_editor_runtime_with_body() {
        let html = "<!DOCTYPE html><html><body><p>Content</p></body></html>";
        let result = inject_editor_runtime(html, "test.revised.html");
        assert!(result.contains("data-editable-engine=\"true\""));
        assert!(result.contains("test.revised.html"));
        assert!(result.ends_with("</body></html>"));
    }

    #[test]
    fn test_inject_editor_runtime_without_body() {
        let html = "<section><p>Fragment</p></section>";
        let result = inject_editor_runtime(html, "frag.revised.html");
        assert!(result.contains("data-editable-engine=\"true\""));
        assert!(result.starts_with("<section><p>Fragment</p></section>"));
    }

    #[test]
    fn test_derive_file_paths() {
        let input = Path::new("tests/sample.html");
        let (editable_path, revised_filename) = derive_file_paths(input);
        assert_eq!(editable_path, Path::new("tests/sample.editable.html"));
        assert_eq!(revised_filename, "sample.revised.html");

        let svg_input = Path::new("tests/sample.svg");
        let (editable_svg, revised_svg) = derive_file_paths(svg_input);
        assert_eq!(editable_svg, Path::new("tests/sample.editable.html"));
        assert_eq!(revised_svg, "sample.revised.svg");
    }
}
