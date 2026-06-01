use crate::{
    model::{GraphChunkRecord, GraphFileRecord, GraphRelationRecord, GraphSymbolRecord},
    parser_registry::{
        parse_with_tree_sitter, parser_engine_for_language, ParserEngine, TreeSitterParseOutput,
    },
};

pub(crate) fn extract_file_details(
    file: &GraphFileRecord,
    content: &str,
) -> (
    Vec<GraphSymbolRecord>,
    Vec<GraphRelationRecord>,
    Vec<GraphChunkRecord>,
) {
    if parser_engine_for_language(&file.language) == ParserEngine::TreeSitterPreferred {
        if let Ok(Some(parsed)) = parse_with_tree_sitter(&file.language, content) {
            if !parsed.symbols.is_empty() || !parsed.imports.is_empty() {
                return build_from_tree_sitter(file, content, parsed);
            }
        }
    }

    let mut symbols = Vec::new();
    let mut relations = Vec::new();
    let lines = content.lines().collect::<Vec<_>>();
    let mut active_symbols: Vec<(String, usize)> = Vec::new();
    let mut pending_test = false;
    let mut pending_component = false;
    let parser_source = "structured-fallback";

    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if is_test_annotation(&file.language, trimmed) {
            pending_test = true;
        }
        if is_component_annotation(&file.language, trimmed) {
            pending_component = true;
        }

        while active_symbols
            .last()
            .map(|(_, end_line)| *end_line < line_number)
            .unwrap_or(false)
        {
            active_symbols.pop();
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
                source: format!("{parser_source}-import"),
            });
        }

        if let Some(mut extracted) = extract_symbol(file, line, pending_test, pending_component) {
            let parent_symbol_id = active_symbols.last().map(|(id, _)| id.clone());
            if parent_symbol_id.is_some() && extracted.kind == "function" {
                extracted.kind = "method".to_string();
            }
            let end_line = find_symbol_end(&lines, index, &file.language);
            let id = format!("symbol:{}:{}", file.path, symbols.len() + 1);
            symbols.push(GraphSymbolRecord {
                id: id.clone(),
                file_id: file.id.clone(),
                language: file.language.clone(),
                name: extracted.name,
                kind: extracted.kind,
                signature: extracted.signature.clone(),
                start_line: line_number,
                end_line,
                parent_symbol_id: parent_symbol_id.clone(),
                visibility: extract_visibility(line),
                path: file.path.clone(),
            });
            relations.push(GraphRelationRecord {
                id: format!("relation:{}:contains:{}", file.path, symbols.len()),
                from_type: "file".to_string(),
                from_id: file.id.clone(),
                to_type: "symbol".to_string(),
                to_id: id.clone(),
                relation_kind: "contains".to_string(),
                confidence: "high".to_string(),
                source: parser_source.to_string(),
            });
            if let Some(parent_id) = parent_symbol_id {
                relations.push(GraphRelationRecord {
                    id: format!("relation:{}:parent_of:{}", parent_id, id),
                    from_type: "symbol".to_string(),
                    from_id: parent_id,
                    to_type: "symbol".to_string(),
                    to_id: id.clone(),
                    relation_kind: "parent_of".to_string(),
                    confidence: "medium".to_string(),
                    source: format!("{parser_source}-parent"),
                });
            }
            for extends in extracted.extends {
                relations.push(GraphRelationRecord {
                    id: format!("relation:{}:extends:{}:{}", file.path, id, extends),
                    from_type: "symbol".to_string(),
                    from_id: id.clone(),
                    to_type: "symbol".to_string(),
                    to_id: extends,
                    relation_kind: "extends".to_string(),
                    confidence: "low".to_string(),
                    source: format!("{parser_source}-inheritance"),
                });
            }
            for implements in extracted.implements {
                relations.push(GraphRelationRecord {
                    id: format!("relation:{}:implements:{}:{}", file.path, id, implements),
                    from_type: "symbol".to_string(),
                    from_id: id.clone(),
                    to_type: "symbol".to_string(),
                    to_id: implements,
                    relation_kind: "implements".to_string(),
                    confidence: "low".to_string(),
                    source: format!("{parser_source}-inheritance"),
                });
            }
            if end_line > line_number {
                active_symbols.push((id, end_line));
            }
            pending_test = false;
            pending_component = false;
        }
    }

    let chunks = build_chunks(file, content, &symbols);
    (symbols, relations, chunks)
}

fn build_from_tree_sitter(
    file: &GraphFileRecord,
    content: &str,
    parsed: TreeSitterParseOutput,
) -> (
    Vec<GraphSymbolRecord>,
    Vec<GraphRelationRecord>,
    Vec<GraphChunkRecord>,
) {
    let parser_source = if parsed.has_error {
        "tree-sitter-degraded"
    } else {
        "tree-sitter"
    };
    let mut symbols = Vec::new();
    let mut relations = Vec::new();
    let mut symbol_ids = Vec::new();

    for import in parsed.imports {
        relations.push(GraphRelationRecord {
            id: format!("relation:{}:import:{}", file.path, import.line),
            from_type: "file".to_string(),
            from_id: file.id.clone(),
            to_type: "module".to_string(),
            to_id: import.module,
            relation_kind: "imports".to_string(),
            confidence: "high".to_string(),
            source: format!("{parser_source}-import"),
        });
    }

    for (index, candidate) in parsed.symbols.iter().enumerate() {
        let parent_symbol_id = candidate
            .parent_index
            .and_then(|parent_index| symbol_ids.get(parent_index).cloned());
        let signature = candidate.signature.clone().unwrap_or_default();
        let mut kind = candidate.kind.clone();
        if parent_symbol_id.is_some() && kind == "function" {
            kind = "method".to_string();
        }
        if is_tree_sitter_test_symbol(file.language.as_str(), &candidate.name, &signature) {
            kind = "test".to_string();
        }
        if is_tree_sitter_component_symbol(file.language.as_str(), &signature) {
            kind = "component".to_string();
        }
        let id = format!("symbol:{}:{}", file.path, index + 1);
        symbol_ids.push(id.clone());
        symbols.push(GraphSymbolRecord {
            id: id.clone(),
            file_id: file.id.clone(),
            language: file.language.clone(),
            name: candidate.name.clone(),
            kind,
            signature: candidate.signature.clone(),
            start_line: candidate.start_line,
            end_line: candidate.end_line.max(candidate.start_line),
            parent_symbol_id: parent_symbol_id.clone(),
            visibility: candidate
                .visibility
                .clone()
                .or_else(|| candidate.signature.as_deref().and_then(extract_visibility)),
            path: file.path.clone(),
        });
        relations.push(GraphRelationRecord {
            id: format!("relation:{}:contains:{}", file.path, index + 1),
            from_type: "file".to_string(),
            from_id: file.id.clone(),
            to_type: "symbol".to_string(),
            to_id: id.clone(),
            relation_kind: "contains".to_string(),
            confidence: "high".to_string(),
            source: parser_source.to_string(),
        });
        if let Some(parent_id) = parent_symbol_id {
            relations.push(GraphRelationRecord {
                id: format!("relation:{}:parent_of:{}", parent_id, id),
                from_type: "symbol".to_string(),
                from_id: parent_id,
                to_type: "symbol".to_string(),
                to_id: id.clone(),
                relation_kind: "parent_of".to_string(),
                confidence: "high".to_string(),
                source: format!("{parser_source}-parent"),
            });
        }
        for extends in candidate
            .signature
            .as_deref()
            .map(extract_extends)
            .unwrap_or_default()
        {
            relations.push(GraphRelationRecord {
                id: format!("relation:{}:extends:{}:{}", file.path, id, extends),
                from_type: "symbol".to_string(),
                from_id: id.clone(),
                to_type: "symbol".to_string(),
                to_id: extends,
                relation_kind: "extends".to_string(),
                confidence: "medium".to_string(),
                source: format!("{parser_source}-inheritance"),
            });
        }
        for implements in candidate
            .signature
            .as_deref()
            .map(extract_implements)
            .unwrap_or_default()
        {
            relations.push(GraphRelationRecord {
                id: format!("relation:{}:implements:{}:{}", file.path, id, implements),
                from_type: "symbol".to_string(),
                from_id: id.clone(),
                to_type: "symbol".to_string(),
                to_id: implements,
                relation_kind: "implements".to_string(),
                confidence: "medium".to_string(),
                source: format!("{parser_source}-inheritance"),
            });
        }
    }

    let chunks = build_chunks(file, content, &symbols);
    (symbols, relations, chunks)
}

fn is_tree_sitter_test_symbol(language: &str, name: &str, signature: &str) -> bool {
    name.to_ascii_lowercase().starts_with("test")
        || signature.contains("#[test]")
        || signature.contains("@Test")
        || (language == "python" && signature.contains("@pytest"))
}

fn is_tree_sitter_component_symbol(language: &str, signature: &str) -> bool {
    signature.contains("@Composable")
        || signature.contains("React.FC")
        || signature.contains("extends StatelessWidget")
        || signature.contains("extends StatefulWidget")
        || (language == "swift"
            && (signature.contains(": View") || signature.contains("some View")))
}

struct ExtractedSymbol {
    kind: String,
    name: String,
    signature: Option<String>,
    extends: Vec<String>,
    implements: Vec<String>,
}

fn extract_symbol(
    file: &GraphFileRecord,
    line: &str,
    pending_test: bool,
    pending_component: bool,
) -> Option<ExtractedSymbol> {
    let language = file.language.as_str();
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
                return Some(extracted("markdown_heading", name, trimmed));
            }
        }
        return None;
    }

    if let Some(symbol) = extract_mobile_symbol(file, trimmed) {
        return Some(symbol);
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
        ("impl ", "module"),
        ("package ", "package"),
        ("object ", "class"),
        ("data class ", "class"),
        ("async def ", "function"),
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
            return Some(classify_symbol(
                kind,
                name,
                trimmed,
                language,
                pending_test,
                pending_component,
            ));
        }
        if trimmed.starts_with("pub ")
            || trimmed.starts_with("public ")
            || trimmed.starts_with("private ")
            || trimmed.starts_with("protected ")
            || trimmed.starts_with("internal ")
            || trimmed.starts_with("static ")
        {
            if let Some(name) = extract_after_marker(trimmed, marker) {
                return Some(classify_symbol(
                    kind,
                    name,
                    trimmed,
                    language,
                    pending_test,
                    pending_component,
                ));
            }
        }
    }

    if language == "go" && trimmed.starts_with("func (") {
        if let Some(after_receiver) = trimmed.split(')').nth(1) {
            let name = after_receiver
                .trim()
                .split(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
                .next()
                .map(clean_identifier)
                .filter(|name| !name.is_empty())?;
            return Some(classify_symbol(
                "method",
                name,
                trimmed,
                language,
                pending_test,
                pending_component,
            ));
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
                return Some(classify_symbol(
                    "function",
                    name,
                    trimmed,
                    language,
                    pending_test,
                    pending_component,
                ));
            }
        }
    }

    if language == "dart" && (trimmed.contains("Widget build(") || trimmed.contains("Widget ")) {
        return Some(classify_symbol(
            "component",
            "build".to_string(),
            trimmed,
            language,
            pending_test,
            true,
        ));
    }

    None
}

fn extracted(kind: &str, name: &str, signature: &str) -> ExtractedSymbol {
    ExtractedSymbol {
        kind: kind.to_string(),
        name: name.to_string(),
        signature: Some(signature.to_string()),
        extends: Vec::new(),
        implements: Vec::new(),
    }
}

fn classify_symbol(
    kind: &str,
    name: String,
    signature: &str,
    language: &str,
    pending_test: bool,
    pending_component: bool,
) -> ExtractedSymbol {
    let mut symbol = extracted(kind, &name, signature);
    if pending_test
        || name.to_ascii_lowercase().starts_with("test")
        || signature.contains("#[test]")
    {
        symbol.kind = "test".to_string();
    }
    if pending_component
        || signature.contains("@Composable")
        || signature.contains(": View")
        || signature.contains("React.FC")
        || signature.contains("extends StatelessWidget")
        || signature.contains("extends StatefulWidget")
        || (language == "swift" && signature.contains("some View"))
    {
        symbol.kind = "component".to_string();
    }
    symbol.extends = extract_extends(signature);
    symbol.implements = extract_implements(signature);
    symbol
}

fn extract_config_key(trimmed: &str) -> Option<ExtractedSymbol> {
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
    Some(extracted("config_key", key, trimmed))
}

fn extract_mobile_symbol(file: &GraphFileRecord, trimmed: &str) -> Option<ExtractedSymbol> {
    if file.name == "AndroidManifest.xml" {
        for (tag, kind) in [
            ("<activity", "component"),
            ("<service", "component"),
            ("<receiver", "component"),
            ("<provider", "component"),
            ("<uses-permission", "config_key"),
        ] {
            if trimmed.starts_with(tag) {
                let name = extract_xml_name(trimmed)
                    .unwrap_or_else(|| tag.trim_start_matches('<').to_string());
                return Some(extracted(kind, &name, trimmed));
            }
        }
    }

    if file.name == "Info.plist" && trimmed.contains("CFBundleIdentifier") {
        return Some(extracted("config_key", "CFBundleIdentifier", trimmed));
    }
    if file.name == "pubspec.yaml"
        && (trimmed.starts_with("name:") || trimmed.starts_with("assets:"))
    {
        return extract_config_key(trimmed);
    }
    None
}

fn extract_xml_name(trimmed: &str) -> Option<String> {
    for key in ["android:name=", "name="] {
        let start = trimmed.find(key)? + key.len();
        let value = trimmed[start..].trim_start();
        let quote = value.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let rest = &value[1..];
        let end = rest.find(quote)?;
        return Some(rest[..end].to_string());
    }
    None
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

fn is_test_annotation(language: &str, trimmed: &str) -> bool {
    matches!(
        trimmed,
        "#[test]" | "@Test" | "@org.junit.Test" | "@pytest.mark" | "@testable"
    ) || (language == "python" && trimmed.starts_with("@pytest"))
        || (language == "swift" && trimmed.contains("XCTest"))
}

fn is_component_annotation(language: &str, trimmed: &str) -> bool {
    trimmed == "@Composable"
        || trimmed.starts_with("@Composable")
        || trimmed.contains("@ReactComponent")
        || (language == "swift" && trimmed.contains("@ViewBuilder"))
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

fn find_symbol_end(lines: &[&str], start_index: usize, language: &str) -> usize {
    let start_line = lines[start_index];
    if language == "python" {
        return find_indent_block_end(lines, start_index);
    }
    if !start_line.contains('{')
        && matches!(
            language,
            "markdown" | "json" | "yaml" | "toml" | "xml" | "plist" | "gradle"
        )
    {
        return start_index + 1;
    }

    let mut balance = 0isize;
    let mut saw_open = false;
    for (index, line) in lines.iter().enumerate().skip(start_index) {
        for ch in line.chars() {
            if ch == '{' {
                balance += 1;
                saw_open = true;
            } else if ch == '}' {
                balance -= 1;
            }
        }
        if saw_open && balance <= 0 {
            return index + 1;
        }
        if !saw_open && index > start_index && !line.trim().is_empty() {
            return index;
        }
    }
    lines.len().max(start_index + 1)
}

fn find_indent_block_end(lines: &[&str], start_index: usize) -> usize {
    let start_indent = leading_spaces(lines[start_index]);
    for (index, line) in lines.iter().enumerate().skip(start_index + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if leading_spaces(line) <= start_indent {
            return index;
        }
    }
    lines.len().max(start_index + 1)
}

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn extract_extends(signature: &str) -> Vec<String> {
    let mut values = Vec::new();
    for marker in [" extends ", " : "] {
        if let Some(after) = signature.split(marker).nth(1) {
            let after = after.split(" implements ").next().unwrap_or(after);
            values.extend(
                after
                    .split(|ch| ch == ',' || ch == '{' || ch == '<')
                    .next()
                    .map(clean_identifier)
                    .filter(|value| !value.is_empty()),
            );
        }
    }
    values
}

fn extract_implements(signature: &str) -> Vec<String> {
    let Some(after) = signature.split(" implements ").nth(1) else {
        return Vec::new();
    };
    after
        .split(|ch| ch == ',' || ch == '{')
        .map(clean_identifier)
        .filter(|value| !value.is_empty())
        .collect()
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
