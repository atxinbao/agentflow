use crate::model::{GraphChunkRecord, GraphFileRecord, GraphRelationRecord, GraphSymbolRecord};

pub(crate) fn extract_file_details(
    file: &GraphFileRecord,
    content: &str,
) -> (
    Vec<GraphSymbolRecord>,
    Vec<GraphRelationRecord>,
    Vec<GraphChunkRecord>,
) {
    let mut symbols = Vec::new();
    let mut relations = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if let Some((kind, name, signature)) = extract_symbol(&file.language, line) {
            let id = format!("symbol:{}:{}", file.path, symbols.len() + 1);
            symbols.push(GraphSymbolRecord {
                id: id.clone(),
                file_id: file.id.clone(),
                language: file.language.clone(),
                name,
                kind,
                signature,
                start_line: line_number,
                end_line: line_number,
                parent_symbol_id: None,
                visibility: extract_visibility(line),
                path: file.path.clone(),
            });
            relations.push(GraphRelationRecord {
                id: format!("relation:{}:contains:{}", file.path, symbols.len()),
                from_type: "file".to_string(),
                from_id: file.id.clone(),
                to_type: "symbol".to_string(),
                to_id: id,
                relation_kind: "contains".to_string(),
                confidence: "high".to_string(),
                source: "lightweight-extractor".to_string(),
            });
        }

        if let Some(import_name) = extract_import(&file.language, line) {
            relations.push(GraphRelationRecord {
                id: format!("relation:{}:import:{}", file.path, line_number),
                from_type: "file".to_string(),
                from_id: file.id.clone(),
                to_type: "module".to_string(),
                to_id: import_name,
                relation_kind: "imports".to_string(),
                confidence: "medium".to_string(),
                source: "lightweight-extractor".to_string(),
            });
        }
    }

    let chunks = build_chunks(file, content, &symbols);
    (symbols, relations, chunks)
}

fn extract_symbol(language: &str, line: &str) -> Option<(String, String, Option<String>)> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with('#') && language != "markdown"
    {
        return None;
    }

    if language == "markdown" {
        let level = trimmed.chars().take_while(|ch| *ch == '#').count();
        if (1..=6).contains(&level) {
            let name = trimmed[level..].trim();
            if !name.is_empty() {
                return Some((
                    "markdown_heading".to_string(),
                    name.to_string(),
                    Some(trimmed.to_string()),
                ));
            }
        }
        return None;
    }

    if matches!(
        language,
        "json" | "yaml" | "toml" | "xml" | "plist" | "gradle"
    ) {
        return extract_config_key(trimmed);
    }

    let patterns = [
        ("class ", "class"),
        ("struct ", "struct"),
        ("enum ", "enum"),
        ("interface ", "interface"),
        ("trait ", "trait"),
        ("protocol ", "protocol"),
        ("object ", "class"),
        ("data class ", "class"),
        ("func ", "function"),
        ("fn ", "function"),
        ("def ", "function"),
        ("function ", "function"),
        ("fun ", "function"),
        ("void ", "function"),
        ("const ", "constant"),
        ("let ", "variable"),
        ("var ", "variable"),
        ("type ", "type_alias"),
        ("extension ", "module"),
        ("module ", "module"),
        ("namespace ", "namespace"),
    ];

    for (marker, kind) in patterns {
        if let Some(name) = extract_after_marker(trimmed, marker) {
            return Some((kind.to_string(), name, Some(trimmed.to_string())));
        }
        if trimmed.starts_with("pub ")
            || trimmed.starts_with("public ")
            || trimmed.starts_with("private ")
            || trimmed.starts_with("protected ")
        {
            if let Some(name) = extract_after_marker(trimmed, marker) {
                return Some((kind.to_string(), name, Some(trimmed.to_string())));
            }
        }
    }

    if language == "typescript" || language == "javascript" {
        if trimmed.contains("=>") {
            if let Some(name) = trimmed
                .split('=')
                .next()
                .and_then(|left| left.split_whitespace().last())
                .map(clean_identifier)
                .filter(|name| !name.is_empty())
            {
                return Some(("function".to_string(), name, Some(trimmed.to_string())));
            }
        }
    }

    if language == "dart" && trimmed.contains("Widget build(") {
        return Some((
            "component".to_string(),
            "build".to_string(),
            Some(trimmed.to_string()),
        ));
    }

    None
}

fn extract_config_key(trimmed: &str) -> Option<(String, String, Option<String>)> {
    let separator = if trimmed.contains('=') { '=' } else { ':' };
    let key = trimmed
        .split(separator)
        .next()?
        .trim()
        .trim_matches('"')
        .trim_matches('\'');
    if key.is_empty() || key.starts_with('{') || key.starts_with('[') || key.starts_with('<') {
        return None;
    }
    Some((
        "config_key".to_string(),
        key.to_string(),
        Some(trimmed.to_string()),
    ))
}

fn extract_after_marker(line: &str, marker: &str) -> Option<String> {
    let index = line.find(marker)?;
    let after = &line[index + marker.len()..];
    let name = after
        .split(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '$'))
        .next()
        .map(clean_identifier)
        .filter(|value| !value.is_empty())?;
    Some(name)
}

fn clean_identifier(value: &str) -> String {
    value
        .trim()
        .trim_matches('{')
        .trim_matches('(')
        .trim_matches(':')
        .trim_matches(';')
        .trim()
        .to_string()
}

fn extract_visibility(line: &str) -> Option<String> {
    let trimmed = line.trim();
    for visibility in ["pub", "public", "private", "protected", "internal"] {
        if trimmed.starts_with(visibility) {
            return Some(visibility.to_string());
        }
    }
    None
}

fn extract_import(language: &str, line: &str) -> Option<String> {
    let trimmed = line.trim();
    let import = match language {
        "rust" => trimmed
            .strip_prefix("use ")
            .or_else(|| trimmed.strip_prefix("mod ")),
        "typescript" | "javascript" => {
            if let Some(index) = trimmed.find(" from ") {
                Some(trimmed[index + 6..].trim())
            } else {
                trimmed.strip_prefix("import ")
            }
        }
        "python" => trimmed
            .strip_prefix("import ")
            .or_else(|| trimmed.strip_prefix("from ")),
        "java" | "kotlin" | "swift" | "go" | "dart" => trimmed.strip_prefix("import "),
        "c" | "cpp" | "csharp" | "objc" => trimmed.strip_prefix("#include "),
        "php" => trimmed.strip_prefix("use "),
        "ruby" => trimmed.strip_prefix("require "),
        "shell" | "powershell" => trimmed.strip_prefix("source "),
        _ => None,
    }?;
    let normalized = import
        .trim()
        .trim_end_matches(';')
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('<')
        .trim_matches('>')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn build_chunks(
    file: &GraphFileRecord,
    content: &str,
    symbols: &[GraphSymbolRecord],
) -> Vec<GraphChunkRecord> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    if symbols.is_empty() {
        chunks.push(chunk_from_lines(
            file,
            None,
            &lines,
            1,
            lines.len().min(120),
        ));
        return chunks;
    }

    for symbol in symbols.iter().take(80) {
        let start = symbol.start_line.saturating_sub(3).max(1);
        let end = (symbol.end_line + 12).min(lines.len());
        chunks.push(chunk_from_lines(file, Some(&symbol.id), &lines, start, end));
    }
    chunks
}

fn chunk_from_lines(
    file: &GraphFileRecord,
    symbol_id: Option<&str>,
    lines: &[&str],
    start_line: usize,
    end_line: usize,
) -> GraphChunkRecord {
    let start_index = start_line.saturating_sub(1);
    let end_index = end_line.min(lines.len());
    let text = lines[start_index..end_index].join("\n");
    GraphChunkRecord {
        id: format!(
            "chunk:{}:{}:{}:{}",
            file.path,
            symbol_id.unwrap_or("file"),
            start_line,
            end_line
        ),
        file_id: file.id.clone(),
        symbol_id: symbol_id.map(str::to_string),
        path: file.path.clone(),
        start_line,
        end_line,
        token_estimate: (text.len() / 4).max(1),
        content_hash: stable_hash(text.as_bytes()),
        text,
    }
}

pub(crate) fn stable_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
