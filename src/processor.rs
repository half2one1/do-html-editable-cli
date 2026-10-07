//! # HTML & SVG Text Processor Engine
//!
//! Provides structured text extraction, CSS selector resolution,
//! targeted element editing, search-and-replace, and serialization
//! for static HTML and standalone SVG documents.

use kuchikiki::traits::*;
use kuchikiki::{ElementData, NodeDataRef, NodeRef};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents an editable text element extracted from HTML or SVG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextUnit {
    /// 0-based sequential index (#0, #1, ...)
    pub index: usize,
    /// HTML tag name (e.g. "h1", "p", "li", "text", "tspan")
    pub tag: String,
    /// Best CSS selector for this element
    pub selector: String,
    /// ID attribute if present
    pub id: Option<String>,
    /// Class attribute if present
    pub class: Option<String>,
    /// Current text content (trimmed)
    pub text: String,
}

/// Core processor for inspecting and modifying HTML/SVG documents
pub struct HtmlProcessor {
    pub file_path: PathBuf,
    pub is_svg: bool,
    pub doctype: Option<String>,
    pub document: NodeRef,
}

impl HtmlProcessor {
    /// Loads and parses an HTML or SVG document from a local file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_buf = path.as_ref().to_path_buf();
        if !path_buf.exists() {
            return Err(format!("File '{}' not found.", path_buf.display()));
        }

        let raw_content = fs::read_to_string(&path_buf)
            .map_err(|e| format!("Failed to read '{}': {}", path_buf.display(), e))?;

        Self::from_string(path_buf, raw_content)
    }

    /// Parses raw HTML or SVG content
    pub fn from_string(file_path: PathBuf, raw_content: String) -> Result<Self, String> {
        let is_svg = file_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("svg"))
            .unwrap_or(false)
            || raw_content.trim_start().starts_with("<svg");

        // Extract doctype if present
        let doctype = if let Some(start) = raw_content.find("<!DOCTYPE") {
            if let Some(end) = raw_content[start..].find('>') {
                Some(raw_content[start..start + end + 1].to_string())
            } else {
                None
            }
        } else {
            None
        };

        let document = kuchikiki::parse_html().one(raw_content);

        Ok(Self {
            file_path,
            is_svg,
            doctype,
            document,
        })
    }

    /// Selects candidate editable elements across the document
    fn get_candidate_elements(&self) -> Vec<NodeDataRef<ElementData>> {
        // Query elements that typically contain standalone or block text
        let selector = "h1, h2, h3, h4, h5, h6, p, li, th, td, blockquote, label, figcaption, button, text, tspan";
        if let Ok(matches) = self.document.select(selector) {
            matches
                .filter(|node| {
                    // Filter out our internal toolbar if present
                    let attrs = node.attributes.borrow();
                    if attrs.contains("data-editable-engine") {
                        return false;
                    }
                    if let Some(id) = attrs.get("id") {
                        if id.starts_with("__mhe_") {
                            return false;
                        }
                    }
                    true
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Enumerates all editable text units in the document
    pub fn list_texts(&self, filter: Option<&str>, limit: Option<usize>) -> Vec<TextUnit> {
        let candidates = self.get_candidate_elements();
        let mut results = Vec::new();
        let mut idx = 0;

        for node in candidates {
            let tag = node.name.local.to_string();
            let attrs = node.attributes.borrow();
            let id = attrs.get("id").map(|s| s.to_string());
            let class = attrs.get("class").map(|s| s.to_string());
            let text = node.text_contents();
            let trimmed = text.trim();

            if trimmed.is_empty() {
                continue;
            }

            if let Some(pattern) = filter {
                if !trimmed.to_lowercase().contains(&pattern.to_lowercase()) {
                    continue;
                }
            }

            // Construct readable selector
            let selector = if let Some(ref id_val) = id {
                format!("{}#{}", tag, id_val)
            } else if let Some(ref cls) = class {
                let first_cls = cls.split_whitespace().next().unwrap_or("");
                if !first_cls.is_empty() {
                    format!("{}.{}", tag, first_cls)
                } else {
                    tag.clone()
                }
            } else {
                tag.clone()
            };

            results.push(TextUnit {
                index: idx,
                tag,
                selector,
                id,
                class,
                text: trimmed.to_string(),
            });

            idx += 1;
            if let Some(max) = limit {
                if results.len() >= max {
                    break;
                }
            }
        }

        results
    }

    /// Finds a node by target query:
    /// - Index syntax: `#0`, `#1`, etc.
    /// - ID syntax: `#my-id`
    /// - CSS selector syntax: `h1.title`, `p#desc`, etc.
    fn find_target_node(&self, target: &str) -> Result<NodeDataRef<ElementData>, String> {
        let trimmed = target.trim();

        // 1. Check index format: #0, #12
        if trimmed.starts_with('#') {
            if let Ok(idx) = trimmed[1..].parse::<usize>() {
                let candidates = self.get_candidate_elements();
                let non_empty: Vec<_> = candidates
                    .into_iter()
                    .filter(|n| !n.text_contents().trim().is_empty())
                    .collect();

                if let Some(matched) = non_empty.get(idx) {
                    return Ok(matched.clone());
                } else {
                    return Err(format!(
                        "Index #{} out of bounds (0..{}).",
                        idx,
                        non_empty.len()
                    ));
                }
            }
        }

        // 2. Check CSS selector
        match self.document.select(trimmed) {
            Ok(mut matches) => {
                if let Some(matched) = matches.next() {
                    Ok(matched)
                } else {
                    Err(format!("No element matching selector '{}' found.", trimmed))
                }
            }
            Err(_) => Err(format!("Invalid CSS selector: '{}'.", trimmed)),
        }
    }

    /// Gets the text of an element by target (index or selector)
    pub fn get_text(&self, target: &str) -> Result<String, String> {
        let node = self.find_target_node(target)?;
        Ok(node.text_contents().trim().to_string())
    }

    /// Sets the text content of a target element, replacing its internal children
    pub fn set_text(&self, target: &str, new_text: &str) -> Result<(), String> {
        let node_data = self.find_target_node(target)?;
        let node_ref = node_data.as_node();

        // Detach all current child nodes
        let children: Vec<NodeRef> = node_ref.children().collect();
        for child in children {
            child.detach();
        }

        // Append new text node
        node_ref.append(NodeRef::new_text(new_text));
        Ok(())
    }

    /// Appends text to a target element
    pub fn append_text(&self, target: &str, text_to_append: &str) -> Result<(), String> {
        let node_data = self.find_target_node(target)?;
        let node_ref = node_data.as_node();
        node_ref.append(NodeRef::new_text(text_to_append));
        Ok(())
    }

    /// Prepends text to a target element
    pub fn prepend_text(&self, target: &str, text_to_prepend: &str) -> Result<(), String> {
        let node_data = self.find_target_node(target)?;
        let node_ref = node_data.as_node();
        node_ref.prepend(NodeRef::new_text(text_to_prepend));
        Ok(())
    }

    /// Replaces occurrences of a substring across the entire document or within a target element
    pub fn replace_text(
        &self,
        old_text: &str,
        new_text: &str,
        target: Option<&str>,
    ) -> Result<usize, String> {
        if old_text.is_empty() {
            return Err("Cannot replace an empty search string.".to_string());
        }

        let mut replacements = 0;

        let root_node = if let Some(target_query) = target {
            self.find_target_node(target_query)?.as_node().clone()
        } else {
            self.document.clone()
        };

        for edge in root_node.traverse() {
            if let kuchikiki::iter::NodeEdge::Start(node) = edge {
                // Skip internal engine nodes
                if let Some(element) = node.as_element() {
                    let attrs = element.attributes.borrow();
                    if attrs.contains("data-editable-engine") {
                        continue;
                    }
                }
                if let Some(text_cell) = node.as_text() {
                    let mut text = text_cell.borrow_mut();
                    if text.contains(old_text) {
                        *text = text.replace(old_text, new_text);
                        replacements += 1;
                    }
                }
            }
        }

        Ok(replacements)
    }

    /// Serializes the modified document to a clean string
    pub fn serialize_clean(&self) -> String {
        if self.is_svg {
            // If standalone SVG, serialize only the <svg> root element
            if let Ok(mut svgs) = self.document.select("svg") {
                if let Some(svg_node) = svgs.next() {
                    let svg_content = svg_node.as_node().to_string();
                    return format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", svg_content);
                }
            }
        }

        let doctype_prefix = self
            .doctype
            .as_deref()
            .unwrap_or("<!DOCTYPE html>\n");

        format!("{}{}", doctype_prefix, self.document.to_string())
    }

    /// Saves the modified document to disk
    pub fn save<P: AsRef<Path>>(&self, dest: Option<P>) -> Result<PathBuf, String> {
        let target_path = if let Some(p) = dest {
            p.as_ref().to_path_buf()
        } else {
            self.file_path.clone()
        };

        let content = self.serialize_clean();
        fs::write(&target_path, content)
            .map_err(|e| format!("Failed to write to '{}': {}", target_path.display(), e))?;

        Ok(target_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_and_set_text_html() {
        let html = r#"<!DOCTYPE html><html><body><h1 id="title">Original Title</h1><p>First paragraph.</p></body></html>"#;
        let proc = HtmlProcessor::from_string(PathBuf::from("test.html"), html.to_string()).unwrap();

        let texts = proc.list_texts(None, None);
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0].text, "Original Title");

        proc.set_text("#title", "Revised Title").unwrap();
        assert_eq!(proc.get_text("#title").unwrap(), "Revised Title");

        let serialized = proc.serialize_clean();
        assert!(serialized.contains("Revised Title"));
    }

    #[test]
    fn test_replace_text_across_doc() {
        let html = r#"<div><p>Version 2025</p><span>Report 2025</span></div>"#;
        let proc = HtmlProcessor::from_string(PathBuf::from("test.html"), html.to_string()).unwrap();

        let count = proc.replace_text("2025", "2026", None).unwrap();
        assert!(count >= 1);
        let serialized = proc.serialize_clean();
        assert!(serialized.contains("2026"));
        assert!(!serialized.contains("2025"));
    }

    #[test]
    fn test_svg_processing() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text id="t1">SVG Label</text></svg>"#;
        let proc = HtmlProcessor::from_string(PathBuf::from("panel.svg"), svg.to_string()).unwrap();

        assert!(proc.is_svg);
        let texts = proc.list_texts(None, None);
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].text, "SVG Label");

        proc.set_text("#0", "Updated SVG Label").unwrap();
        assert_eq!(proc.get_text("#0").unwrap(), "Updated SVG Label");
    }
}
