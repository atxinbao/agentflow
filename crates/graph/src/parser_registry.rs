use tree_sitter::{Language, Node, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParserEngine {
    TreeSitterPreferred,
    StructuredFallback,
}

#[derive(Debug, Clone)]
pub(crate) struct TreeSitterParseOutput {
    pub(crate) symbols: Vec<TreeSitterSymbolCandidate>,
    pub(crate) imports: Vec<TreeSitterImportCandidate>,
    pub(crate) has_error: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct TreeSitterSymbolCandidate {
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) signature: Option<String>,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) parent_index: Option<usize>,
    pub(crate) visibility: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct TreeSitterImportCandidate {
    pub(crate) module: String,
    pub(crate) line: usize,
}

pub(crate) fn parser_engine_for_language(language: &str) -> ParserEngine {
    if is_l1_language(language) {
        ParserEngine::TreeSitterPreferred
    } else {
        ParserEngine::StructuredFallback
    }
}

pub(crate) fn is_l1_language(language: &str) -> bool {
    matches!(
        language,
        "typescript"
            | "javascript"
            | "python"
            | "java"
            | "kotlin"
            | "swift"
            | "go"
            | "rust"
            | "c"
            | "cpp"
            | "csharp"
            | "dart"
    )
}

pub(crate) fn parse_with_tree_sitter(
    language: &str,
    content: &str,
) -> Result<Option<TreeSitterParseOutput>, String> {
    let Some(grammar) = grammar_for_language(language) else {
        return Ok(None);
    };

    let mut parser = Parser::new();
    parser
        .set_language(&grammar)
        .map_err(|error| format!("tree-sitter parser unavailable for {language}: {error}"))?;
    let tree = parser
        .parse(content, None)
        .ok_or_else(|| format!("tree-sitter parser returned no tree for {language}"))?;

    let mut output = TreeSitterParseOutput {
        symbols: Vec::new(),
        imports: Vec::new(),
        has_error: tree.root_node().has_error(),
    };
    collect_nodes(
        language,
        content,
        tree.root_node(),
        None,
        &mut output.symbols,
        &mut output.imports,
    );
    Ok(Some(output))
}

fn grammar_for_language(language: &str) -> Option<Language> {
    Some(match language {
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "javascript" => tree_sitter_javascript::LANGUAGE.into(),
        "python" => tree_sitter_python::LANGUAGE.into(),
        "java" => tree_sitter_java::LANGUAGE.into(),
        "kotlin" => tree_sitter_kotlin_ng::LANGUAGE.into(),
        "swift" => tree_sitter_swift::LANGUAGE.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "c" => tree_sitter_c::LANGUAGE.into(),
        "cpp" => tree_sitter_cpp::LANGUAGE.into(),
        "csharp" => tree_sitter_c_sharp::LANGUAGE.into(),
        "dart" => tree_sitter_dart::LANGUAGE.into(),
        _ => return None,
    })
}

fn collect_nodes(
    language: &str,
    content: &str,
    node: Node<'_>,
    parent_index: Option<usize>,
    symbols: &mut Vec<TreeSitterSymbolCandidate>,
    imports: &mut Vec<TreeSitterImportCandidate>,
) {
    let mut next_parent = parent_index;
    if let Some(module) = import_module(language, content, node) {
        imports.push(TreeSitterImportCandidate {
            module,
            line: node.start_position().row + 1,
        });
    }

    if let Some((kind, name)) = symbol_kind_and_name(language, content, node) {
        let signature = node
            .utf8_text(content.as_bytes())
            .ok()
            .and_then(|text| text.lines().next().map(str::trim).map(str::to_string))
            .filter(|text| !text.is_empty());
        symbols.push(TreeSitterSymbolCandidate {
            kind,
            name,
            signature,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row.saturating_add(1),
            parent_index,
            visibility: visibility_from_node(content, node),
        });
        next_parent = Some(symbols.len() - 1);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nodes(language, content, child, next_parent, symbols, imports);
    }
}

fn symbol_kind_and_name(language: &str, content: &str, node: Node<'_>) -> Option<(String, String)> {
    let kind = symbol_kind(language, node.kind())?;
    let name_node = node.child_by_field_name("name").or_else(|| {
        if matches!(node.kind(), "impl_item" | "type_declaration") {
            node.child_by_field_name("type")
        } else {
            None
        }
    })?;
    let name = name_node
        .utf8_text(content.as_bytes())
        .ok()
        .map(clean_name)
        .filter(|name| !name.is_empty())?;
    Some((kind.to_string(), name))
}

fn symbol_kind(language: &str, node_kind: &str) -> Option<&'static str> {
    match (language, node_kind) {
        ("rust", "function_item") => Some("function"),
        ("rust", "struct_item") => Some("struct"),
        ("rust", "enum_item") => Some("enum"),
        ("rust", "trait_item") => Some("trait"),
        ("rust", "impl_item") => Some("module"),
        ("rust", "mod_item") => Some("module"),

        ("typescript" | "javascript", "function_declaration") => Some("function"),
        ("typescript" | "javascript", "class_declaration") => Some("class"),
        ("typescript", "interface_declaration") => Some("interface"),
        ("typescript", "type_alias_declaration") => Some("type_alias"),
        ("typescript" | "javascript", "method_definition") => Some("method"),
        ("typescript" | "javascript", "lexical_declaration") => Some("variable"),

        ("python", "function_definition") => Some("function"),
        ("python", "class_definition") => Some("class"),
        ("python", "decorated_definition") => Some("function"),

        ("java", "class_declaration") => Some("class"),
        ("java", "interface_declaration") => Some("interface"),
        ("java", "enum_declaration") => Some("enum"),
        ("java", "record_declaration") => Some("class"),
        ("java", "method_declaration") => Some("method"),

        ("kotlin", "class_declaration") => Some("class"),
        ("kotlin", "object_declaration") => Some("class"),
        ("kotlin", "function_declaration") => Some("function"),

        ("swift", "class_declaration") => Some("class"),
        ("swift", "struct_declaration") => Some("struct"),
        ("swift", "enum_declaration") => Some("enum"),
        ("swift", "protocol_declaration") => Some("protocol"),
        ("swift", "extension_declaration") => Some("module"),
        ("swift", "function_declaration") => Some("function"),

        ("go", "function_declaration") => Some("function"),
        ("go", "method_declaration") => Some("method"),
        ("go", "type_spec") => Some("type_alias"),

        ("c" | "cpp", "function_definition") => Some("function"),
        ("cpp", "class_specifier") => Some("class"),
        ("c" | "cpp", "struct_specifier") => Some("struct"),
        ("cpp", "namespace_definition") => Some("namespace"),

        ("csharp", "class_declaration") => Some("class"),
        ("csharp", "interface_declaration") => Some("interface"),
        ("csharp", "method_declaration") => Some("method"),
        ("csharp", "namespace_declaration") => Some("namespace"),

        ("dart", "class_definition") => Some("class"),
        ("dart", "mixin_declaration") => Some("type_alias"),
        ("dart", "extension_declaration") => Some("module"),
        ("dart", "function_signature") => Some("function"),
        ("dart", "method_signature") => Some("method"),

        _ => None,
    }
}

fn import_module(language: &str, content: &str, node: Node<'_>) -> Option<String> {
    let import_node = match (language, node.kind()) {
        ("rust", "use_declaration") | ("rust", "mod_item") => Some(node),
        ("typescript" | "javascript", "import_statement") => Some(node),
        ("python", "import_statement") | ("python", "import_from_statement") => Some(node),
        ("java" | "kotlin" | "swift" | "go" | "dart", "import_declaration") => Some(node),
        ("go", "package_clause") => Some(node),
        ("c" | "cpp", "preproc_include") => Some(node),
        ("csharp", "using_directive") => Some(node),
        ("java", "package_declaration") => Some(node),
        _ => None,
    }?;
    let text = import_node.utf8_text(content.as_bytes()).ok()?.trim();
    normalize_import_text(language, text)
}

fn normalize_import_text(language: &str, text: &str) -> Option<String> {
    let value = match language {
        "rust" => text
            .strip_prefix("use ")
            .or_else(|| text.strip_prefix("mod "))
            .unwrap_or(text),
        "typescript" | "javascript" => text
            .split(" from ")
            .nth(1)
            .or_else(|| text.strip_prefix("import "))
            .unwrap_or(text),
        "python" => text
            .strip_prefix("import ")
            .or_else(|| text.strip_prefix("from "))
            .unwrap_or(text),
        "java" | "kotlin" | "swift" | "go" | "dart" => text
            .strip_prefix("import ")
            .or_else(|| text.strip_prefix("package "))
            .unwrap_or(text),
        "c" | "cpp" => text.strip_prefix("#include ").unwrap_or(text),
        "csharp" => text.strip_prefix("using ").unwrap_or(text),
        _ => text,
    };
    let normalized = value
        .trim()
        .trim_start_matches("static ")
        .trim_end_matches(';')
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('<')
        .trim_matches('>')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn visibility_from_node(content: &str, node: Node<'_>) -> Option<String> {
    let text = node.utf8_text(content.as_bytes()).ok()?.trim_start();
    for visibility in ["pub", "public", "private", "protected", "internal"] {
        if text.starts_with(visibility) {
            return Some(visibility.to_string());
        }
    }
    None
}

fn clean_name(value: &str) -> String {
    value
        .trim()
        .trim_matches('{')
        .trim_matches('(')
        .trim_matches(':')
        .trim_matches(';')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l1_languages_are_registered_for_tree_sitter_preferred_parsing() {
        for language in [
            "typescript",
            "javascript",
            "python",
            "java",
            "kotlin",
            "swift",
            "go",
            "rust",
            "c",
            "cpp",
            "csharp",
            "dart",
        ] {
            assert_eq!(
                parser_engine_for_language(language),
                ParserEngine::TreeSitterPreferred
            );
        }
    }

    #[test]
    fn l1_languages_can_load_tree_sitter_grammar() {
        for language in [
            "typescript",
            "javascript",
            "python",
            "java",
            "kotlin",
            "swift",
            "go",
            "rust",
            "c",
            "cpp",
            "csharp",
            "dart",
        ] {
            let parsed = parse_with_tree_sitter(language, "class Example {}")
                .expect("tree-sitter parse should not fail for registered language");
            assert!(parsed.is_some(), "{language} should have grammar");
        }
    }
}
