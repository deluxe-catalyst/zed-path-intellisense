use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub type Uri = String;

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub name: String,
    pub full_path: PathBuf,
    pub is_directory: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text_format: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<CompletionTextEdit>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionTextEdit {
    pub range: Range,
    pub new_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

pub struct PathCompletionProvider {
    documents: Arc<Mutex<HashMap<Uri, String>>>,
    cache: Arc<Mutex<HashMap<PathBuf, Vec<PathEntry>>>>,
}

impl PathCompletionProvider {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update_document(&self, uri: Uri, text: String) {
        self.documents.lock().unwrap().insert(uri, text);
    }

    pub fn remove_document(&self, uri: &Uri) {
        self.documents.lock().unwrap().remove(uri);
    }

    pub fn uri_to_path(uri: &Uri) -> PathBuf {
        if let Some(rest) = uri.strip_prefix("file://") {
            PathBuf::from(rest)
        } else {
            PathBuf::from(uri)
        }
    }

    pub fn provide_completions(
        &self,
        uri: &Uri,
        position: Position,
        workspace_root: Option<&Path>,
    ) -> Vec<CompletionItem> {
        let text = self.documents.lock().unwrap().get(uri).cloned().unwrap_or_default();

        let (partial_path, is_import, quote_char) = self.extract_path_context(&text, position);

        if partial_path.is_empty() && !is_import {
            return vec![];
        }

        let base_dir = self.determine_base_dir(uri, &partial_path, workspace_root);
        let mut entries = self.scan_directory(&base_dir);

        let pn = partial_path.rsplit('/').next().unwrap_or(&partial_path);
        let is_deep = partial_path.contains('/');

        let mut extra_entries = Vec::new();
        if !is_deep || pn.is_empty() || pn == "." {
            for entry in &entries {
                if entry.is_directory {
                    let subdir = base_dir.join(&entry.name);
                    let sub_entries = self.scan_directory(&subdir);
                    for sub in sub_entries.iter().take(30) {
                        extra_entries.push(PathEntry {
                            name: format!("{}/{}", entry.name, sub.name),
                            full_path: sub.full_path.clone(),
                            is_directory: sub.is_directory,
                        });
                    }
                }
            }
        }
        entries.extend(extra_entries);

        let filtered: Vec<PathEntry> = entries
            .into_iter()
            .filter(|e| {
                if pn.is_empty() || pn == "." {
                    true
                } else {
                    let entry_name = e.name.rsplit('/').last().unwrap_or(&e.name);
                    entry_name.starts_with(pn)
                }
            })
            .collect();

        filtered
            .into_iter()
            .map(|entry| self.create_completion_item(entry, &partial_path, is_import, quote_char, position))
            .collect()
    }

    fn extract_path_context(&self, text: &str, position: Position) -> (String, bool, Option<char>) {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return (String::new(), false, None);
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return (String::new(), false, None);
        }

        let before_cursor = &line[..char_pos];

        let quote_char = before_cursor.chars().rev().find(|c| *c == '"' || *c == '\'');

        let is_import = before_cursor.contains("import")
            || before_cursor.contains("require")
            || before_cursor.contains("from");

        let partial = if let Some(quote) = quote_char {
            let after_quote = before_cursor.split(quote).last().unwrap_or("");
            after_quote.to_string()
        } else {
            let parts: Vec<&str> = before_cursor.split([' ', '\t', '(', '{', '[', ',', ';']).collect();
            parts.last().unwrap_or(&"").to_string()
        };

        (partial, is_import, quote_char)
    }

    fn determine_base_dir(&self, uri: &Uri, partial_path: &str, workspace_root: Option<&Path>) -> PathBuf {
        let current_file = Self::uri_to_path(uri);
        let current_dir = current_file.parent().unwrap_or(Path::new("."));

        let target = if partial_path.starts_with('/') {
            if let Some(root) = workspace_root {
                root.join(&partial_path[1..])
            } else {
                PathBuf::from("/")
            }
        } else if partial_path.starts_with("./") || partial_path.starts_with("../") {
            current_dir.join(partial_path)
        } else if partial_path == "." || partial_path.is_empty() {
            current_dir.to_path_buf()
        } else if let Some(root) = workspace_root {
            root.join(partial_path)
        } else {
            current_dir.join(partial_path)
        };

        if target.as_os_str().is_empty() || target.is_dir() || partial_path.ends_with('/') {
            target
        } else {
            target.parent().unwrap_or(&target).to_path_buf()
        }
    }

    fn scan_directory(&self, dir: &Path) -> Vec<PathEntry> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(dir) {
                return cached.clone();
            }
        }

        let mut entries = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return entries;
        }

        for entry in WalkDir::new(dir).min_depth(1).max_depth(1) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let is_directory = entry.file_type().is_dir();

            entries.push(PathEntry {
                name,
                full_path: entry.path().to_path_buf(),
                is_directory,
            });
        }

        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        self.cache.lock().unwrap().insert(dir.to_path_buf(), entries.clone());

        entries
    }

    fn create_completion_item(
        &self,
        entry: PathEntry,
        partial_path: &str,
        _is_import: bool,
        _quote_char: Option<char>,
        position: Position,
    ) -> CompletionItem {
        let name = entry.name;
        let mut label = name.clone();
        if entry.is_directory {
            label.push('/');
        }

        let kind = if entry.is_directory { Some(19u32) } else { Some(18u32) };

        let detail = if entry.is_directory {
            Some("Directory".to_string())
        } else {
            Some("File".to_string())
        };

        let path_part = partial_path.rsplit('/').next().unwrap_or(partial_path);

        let text_edit = Some(CompletionTextEdit {
            range: Range {
                start: Position {
                    line: position.line,
                    character: position.character.saturating_sub(path_part.len() as u32),
                },
                end: position,
            },
            new_text: name.clone(),
        });

        CompletionItem {
            label,
            kind,
            detail,
            insert_text: Some(name.clone()),
            insert_text_format: Some(1),
            text_edit,
        }
    }
}

impl Default for PathCompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}
