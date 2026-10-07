//! # CLI-Based Editing Session (REPL & Batch Script Runner)
//!
//! Provides an interactive and scriptable word-processing session
//! for AI agents and CLI users to view, edit, replace, and export
//! text content in static HTML and SVG files.

use crate::processor::HtmlProcessor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Response envelope for JSON-formatted session outputs
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse<T: Serialize> {
    pub success: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> SessionResponse<T> {
    pub fn ok(command: &str, message: &str, data: Option<T>) -> Self {
        Self {
            success: true,
            command: command.to_string(),
            message: Some(message.to_string()),
            data,
        }
    }

    pub fn err(command: &str, message: &str) -> Self {
        Self {
            success: false,
            command: command.to_string(),
            message: Some(message.to_string()),
            data: None,
        }
    }
}

/// Editing session runner
pub struct EditingSession {
    pub processor: HtmlProcessor,
    pub in_place: bool,
    pub json_mode: bool,
    pub modified: bool,
}

impl EditingSession {
    pub fn new<P: AsRef<Path>>(path: P, in_place: bool, json_mode: bool) -> Result<Self, String> {
        let processor = HtmlProcessor::from_file(path)?;
        Ok(Self {
            processor,
            in_place,
            json_mode,
            modified: false,
        })
    }

    /// Prints output respecting JSON or human-readable mode
    fn emit<T: Serialize>(&self, response: SessionResponse<T>) {
        if self.json_mode {
            let json_str = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
            println!("{}", json_str);
        } else {
            if response.success {
                if let Some(ref msg) = response.message {
                    println!("{}", msg);
                }
            } else {
                if let Some(ref msg) = response.message {
                    eprintln!("Error [{}]: {}", response.command, msg);
                }
            }
        }
    }

    /// Executes a single line command
    pub fn execute_command(&mut self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            return true; // Ignore empty lines or comments
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "exit" | "quit" | "q" => {
                if self.json_mode {
                    self.emit(SessionResponse::<()>::ok("quit", "Session ended.", None));
                } else {
                    println!("Exiting editing session.");
                }
                return false;
            }

            "list" | "ls" => {
                let filter = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                let texts = self.processor.list_texts(filter.as_deref(), None);

                if self.json_mode {
                    self.emit(SessionResponse::ok(
                        "list",
                        &format!("Found {} text elements.", texts.len()),
                        Some(texts),
                    ));
                } else {
                    println!("--- Editable Texts (Total: {}) ---", texts.len());
                    for t in &texts {
                        let id_tag = t.id.as_deref().map(|id| format!(" #{}", id)).unwrap_or_default();
                        println!("[#{:<3}] <{}{}>: {}", t.index, t.tag, id_tag, t.text);
                    }
                    println!("------------------------------------");
                }
            }

            "get" => {
                if parts.len() < 2 {
                    self.emit(SessionResponse::<()>::err("get", "Usage: get <#idx|selector>"));
                    return true;
                }
                let target = parts[1];
                match self.processor.get_text(target) {
                    Ok(text) => {
                        let text_val = text.clone();
                        self.emit(SessionResponse::ok("get", &text, Some(text_val)));
                    }
                    Err(e) => {
                        self.emit(SessionResponse::<()>::err("get", &e));
                    }
                }
            }

            "set" => {
                if parts.len() < 3 {
                    self.emit(SessionResponse::<()>::err(
                        "set",
                        "Usage: set <#idx|selector> <new_text>",
                    ));
                    return true;
                }
                let target = parts[1];
                let remainder = trimmed[cmd.len()..].trim_start();
                let new_text = remainder[target.len()..].trim_start();
                let clean_text = clean_quotes(new_text);

                match self.processor.set_text(target, &clean_text) {
                    Ok(()) => {
                        self.modified = true;
                        self.emit(SessionResponse::ok(
                            "set",
                            &format!("Updated '{}' -> \"{}\"", target, clean_text),
                            Some(clean_text),
                        ));
                    }
                    Err(e) => {
                        self.emit(SessionResponse::<()>::err("set", &e));
                    }
                }
            }

            "append" => {
                if parts.len() < 3 {
                    self.emit(SessionResponse::<()>::err(
                        "append",
                        "Usage: append <#idx|selector> <text_to_append>",
                    ));
                    return true;
                }
                let target = parts[1];
                let remainder = trimmed[cmd.len()..].trim_start();
                let text_to_append = remainder[target.len()..].trim_start();
                let clean_text = clean_quotes(text_to_append);

                match self.processor.append_text(target, &clean_text) {
                    Ok(()) => {
                        self.modified = true;
                        self.emit(SessionResponse::ok(
                            "append",
                            &format!("Appended text to '{}'", target),
                            Some(clean_text),
                        ));
                    }
                    Err(e) => {
                        self.emit(SessionResponse::<()>::err("append", &e));
                    }
                }
            }

            "prepend" => {
                if parts.len() < 3 {
                    self.emit(SessionResponse::<()>::err(
                        "prepend",
                        "Usage: prepend <#idx|selector> <text_to_prepend>",
                    ));
                    return true;
                }
                let target = parts[1];
                let remainder = trimmed[cmd.len()..].trim_start();
                let text_to_prepend = remainder[target.len()..].trim_start();
                let clean_text = clean_quotes(text_to_prepend);

                match self.processor.prepend_text(target, &clean_text) {
                    Ok(()) => {
                        self.modified = true;
                        self.emit(SessionResponse::ok(
                            "prepend",
                            &format!("Prepended text to '{}'", target),
                            Some(clean_text),
                        ));
                    }
                    Err(e) => {
                        self.emit(SessionResponse::<()>::err("prepend", &e));
                    }
                }
            }

            "replace" => {
                // Supports:
                // replace <old> <new>  (global)
                // replace in <#idx|sel> <old> <new>
                let (target_opt, old_text, new_text) = if parts.len() >= 4 && parts[1] == "in" {
                    let target = parts[2];
                    let old_val = parts[3];
                    let remainder = parts[4..].join(" ");
                    (Some(target), old_val.to_string(), clean_quotes(&remainder))
                } else if parts.len() >= 3 {
                    let old_val = parts[1];
                    let remainder = parts[2..].join(" ");
                    (None, old_val.to_string(), clean_quotes(&remainder))
                } else {
                    self.emit(SessionResponse::<()>::err(
                        "replace",
                        "Usage: replace <old> <new> OR replace in <target> <old> <new>",
                    ));
                    return true;
                };

                let clean_old = clean_quotes(&old_text);
                match self.processor.replace_text(&clean_old, &new_text, target_opt) {
                    Ok(count) => {
                        if count > 0 {
                            self.modified = true;
                        }
                        self.emit(SessionResponse::ok(
                            "replace",
                            &format!(
                                "Replaced {} occurrence(s) of '{}' with '{}'.",
                                count, clean_old, new_text
                            ),
                            Some(count),
                        ));
                    }
                    Err(e) => {
                        self.emit(SessionResponse::<()>::err("replace", &e));
                    }
                }
            }

            "status" => {
                let status_info = serde_json::json!({
                    "file": self.processor.file_path.display().to_string(),
                    "is_svg": self.processor.is_svg,
                    "modified": self.modified,
                    "in_place": self.in_place,
                });
                self.emit(SessionResponse::ok(
                    "status",
                    &format!(
                        "File: {} | SVG: {} | Modified: {}",
                        self.processor.file_path.display(),
                        self.processor.is_svg,
                        self.modified
                    ),
                    Some(status_info),
                ));
            }

            "save" => {
                let dest = if parts.len() > 1 {
                    Some(PathBuf::from(parts[1]))
                } else if self.in_place {
                    None
                } else {
                    // Default to .revised.html / .revised.svg
                    let stem = self
                        .processor
                        .file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("document");
                    let ext = if self.processor.is_svg { "svg" } else { "html" };
                    let parent = self
                        .processor
                        .file_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."));
                    Some(parent.join(format!("{}.revised.{}", stem, ext)))
                };

                match self.processor.save(dest.as_ref()) {
                    Ok(saved_path) => {
                        self.modified = false;
                        self.emit(SessionResponse::ok(
                            "save",
                            &format!("Document saved to '{}'", saved_path.display()),
                            Some(saved_path.display().to_string()),
                        ));
                    }
                    Err(e) => {
                        self.emit(SessionResponse::<()>::err("save", &e));
                    }
                }
            }

            "help" => {
                let help_text = "\
Available Session Commands:
  list [filter]                   - Enumerate editable text elements (#0, #1, ...)
  get <#idx|sel>                  - Get element's current text
  set <#idx|sel> <new_text>       - Overwrite element's text
  append <#idx|sel> <text>        - Append text to element
  prepend <#idx|sel> <text>       - Prepend text to element
  replace <old> <new>             - Global search and replace
  replace in <#idx|sel> <old> <new>- Targeted search and replace
  status                          - Show session status
  save [destination_path]         - Save revised document to disk
  quit / exit                     - Exit session";
                if self.json_mode {
                    self.emit(SessionResponse::ok("help", help_text, Some(help_text)));
                } else {
                    println!("{}", help_text);
                }
            }

            _ => {
                self.emit(SessionResponse::<()>::err(
                    &cmd,
                    &format!("Unknown command '{}'. Type 'help' for available commands.", cmd),
                ));
            }
        }

        true
    }

    /// Runs interactive REPL session over stdin
    pub fn run_interactive(&mut self) -> Result<(), String> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();

        if !self.json_mode {
            println!("============================================================");
            println!(" do-html-editable-cli - Interactive Word-Processing Session");
            println!("============================================================");
            println!(" Target File: {}", self.processor.file_path.display());
            println!(" Type 'help' for commands, 'quit' to exit.");
            println!("------------------------------------------------------------");
        }

        let mut line = String::new();
        loop {
            if !self.json_mode {
                print!("editor> ");
                io::stdout().flush().ok();
            }

            line.clear();
            if reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
                break; // EOF
            }

            if !self.execute_command(&line) {
                break;
            }
        }

        Ok(())
    }

    /// Runs a batch script file containing one command per line
    pub fn run_script<P: AsRef<Path>>(&mut self, script_path: P) -> Result<(), String> {
        let path = script_path.as_ref();
        let script_content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read script '{}': {}", path.display(), e))?;

        for line in script_content.lines() {
            if !self.execute_command(line) {
                break;
            }
        }

        Ok(())
    }
}

/// Helper to strip leading and trailing quotes if user quoted their argument
fn clean_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}
