use crate::{
    model::{GraphFileRecord, GraphIndex, GraphRelationRecord},
    parser::{extract_file_details, stable_hash},
};
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

const READ_LIMIT_BYTES: u64 = 1024 * 1024;

pub(crate) fn scan_project(root: &Path) -> Result<GraphIndex> {
    let canonical_root = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;
    let mut paths = Vec::new();
    collect_files(&canonical_root, &canonical_root, &mut paths)?;
    paths.sort();

    let mut files = Vec::new();
    let mut symbols = Vec::new();
    let mut relations = Vec::new();
    let mut chunks = Vec::new();
    let mut file_text_by_id = BTreeMap::new();

    for path in paths {
        let record = build_file_record(&canonical_root, &path)?;
        if record.is_source || record.is_doc || record.is_config {
            let content = read_text_preview(&path).unwrap_or_default();
            let (mut file_symbols, mut file_relations, mut file_chunks) =
                extract_file_details(&record, &content);
            file_text_by_id.insert(record.id.clone(), content);
            symbols.append(&mut file_symbols);
            relations.append(&mut file_relations);
            chunks.append(&mut file_chunks);
        }
        files.push(record);
    }

    relations.extend(build_test_relations(&files));
    relations.extend(build_config_relations(&files));
    relations.extend(build_same_directory_relations(&files));

    Ok(GraphIndex {
        files,
        symbols,
        relations,
        chunks,
    })
}

fn collect_files(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if should_skip_entry(file_name) {
            continue;
        }
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_files(root, &path, paths)?;
        } else if metadata.is_file() {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if !is_ignored_relative(relative) {
                paths.push(path);
            }
        }
    }
    Ok(())
}

fn should_skip_entry(file_name: &str) -> bool {
    matches!(
        file_name,
        ".git"
            | ".agentflow"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | ".cache"
            | "vendor"
            | ".idea"
            | ".vscode"
            | ".DS_Store"
    )
}

fn is_ignored_relative(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(value) = component else {
            return false;
        };
        should_skip_entry(value.to_str().unwrap_or(""))
    })
}

fn build_file_record(root: &Path, path: &Path) -> Result<GraphFileRecord> {
    let metadata = fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
    let relative_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_string();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());
    let language = detect_language(&name, extension.as_deref());
    let is_test = is_test_file(&relative_path, &name);
    let is_doc = language == "markdown";
    let is_config = is_config_file(&name, &language);
    let is_source = is_source_language(&language);
    let is_generated = is_generated_file(&relative_path, &name);
    let kind = file_kind(
        is_source,
        is_test,
        is_doc,
        is_config,
        is_generated,
        extension.as_deref(),
    );
    let bytes = fs::read(path).unwrap_or_default();
    let line_count = if looks_binary(&bytes) {
        0
    } else {
        String::from_utf8_lossy(&bytes).lines().count()
    };
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());

    Ok(GraphFileRecord {
        id: format!("file:{relative_path}"),
        path: relative_path,
        name,
        extension,
        language,
        kind,
        size_bytes: metadata.len(),
        line_count,
        modified_at,
        content_hash: stable_hash(&bytes),
        is_source,
        is_test,
        is_doc,
        is_config,
        is_generated,
        is_ignored: false,
    })
}

fn read_text_preview(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    use std::io::Read;
    file.take(READ_LIMIT_BYTES).read_to_end(&mut bytes)?;
    if metadata.len() > READ_LIMIT_BYTES || looks_binary(&bytes) {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8192).any(|byte| *byte == 0)
}

pub(crate) fn detect_language(name: &str, extension: Option<&str>) -> String {
    let lower_name = name.to_ascii_lowercase();
    match lower_name.as_str() {
        "cargo.toml" | "pyproject.toml" => return "toml".to_string(),
        "package.json" => return "json".to_string(),
        "go.mod" => return "go".to_string(),
        "pubspec.yaml" | "docker-compose.yaml" | "docker-compose.yml" => return "yaml".to_string(),
        "podfile" | "gemfile" => return "ruby".to_string(),
        "package.swift" => return "swift".to_string(),
        "dockerfile" => return "dockerfile".to_string(),
        "androidmanifest.xml" => return "xml".to_string(),
        "info.plist" => return "plist".to_string(),
        "build.gradle" | "settings.gradle" => return "gradle".to_string(),
        "build.gradle.kts" | "settings.gradle.kts" => return "kotlin".to_string(),
        _ => {}
    }

    match extension.unwrap_or("") {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "py" => "python",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "go" => "go",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" | "mm" => "cpp",
        "cs" => "csharp",
        "dart" => "dart",
        "m" => "objc",
        "php" => "php",
        "rb" => "ruby",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        "ps1" => "powershell",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" => "css",
        "md" | "mdx" => "markdown",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" | "xib" | "storyboard" => "xml",
        "plist" | "entitlements" => "plist",
        "gradle" => "gradle",
        _ => "unknown",
    }
    .to_string()
}

fn is_source_language(language: &str) -> bool {
    matches!(
        language,
        "rust"
            | "typescript"
            | "javascript"
            | "python"
            | "java"
            | "kotlin"
            | "swift"
            | "go"
            | "c"
            | "cpp"
            | "csharp"
            | "dart"
            | "objc"
            | "php"
            | "ruby"
            | "sql"
            | "shell"
            | "powershell"
            | "html"
            | "css"
    )
}

fn is_config_file(name: &str, language: &str) -> bool {
    matches!(
        language,
        "json" | "yaml" | "toml" | "xml" | "plist" | "gradle" | "dockerfile"
    ) || matches!(
        name.to_ascii_lowercase().as_str(),
        "cargo.toml"
            | "package.json"
            | "pyproject.toml"
            | "go.mod"
            | "pubspec.yaml"
            | "podfile"
            | "package.swift"
            | "androidmanifest.xml"
            | "info.plist"
    )
}

fn is_test_file(path: &str, name: &str) -> bool {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = name.to_ascii_lowercase();
    lower_path.contains("/test/")
        || lower_path.contains("/tests/")
        || lower_path.contains("__tests__")
        || lower_name.contains("_test.")
        || lower_name.contains(".test.")
        || lower_name.contains(".spec.")
        || lower_name.ends_with("test.rs")
        || lower_name.ends_with("tests.rs")
}

fn is_generated_file(path: &str, name: &str) -> bool {
    let lower_path = path.to_ascii_lowercase();
    let lower_name = name.to_ascii_lowercase();
    lower_path.contains("/generated/")
        || lower_path.contains("/gen/")
        || lower_name.ends_with(".generated.ts")
        || lower_name.ends_with(".g.dart")
        || lower_name.ends_with(".pb.go")
}

fn file_kind(
    is_source: bool,
    is_test: bool,
    is_doc: bool,
    is_config: bool,
    is_generated: bool,
    extension: Option<&str>,
) -> String {
    if is_generated {
        "generated"
    } else if is_test {
        "test"
    } else if is_doc {
        "doc"
    } else if is_config {
        "config"
    } else if is_source {
        "source"
    } else if matches!(
        extension,
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "pdf")
    ) {
        "asset"
    } else {
        "unknown"
    }
    .to_string()
}

fn build_test_relations(files: &[GraphFileRecord]) -> Vec<GraphRelationRecord> {
    let sources: BTreeMap<String, &GraphFileRecord> = files
        .iter()
        .filter(|file| file.is_source && !file.is_test)
        .map(|file| (source_stem(&file.name), file))
        .collect();
    files
        .iter()
        .filter(|file| file.is_test)
        .filter_map(|test| {
            let stem = source_stem(&test.name)
                .replace("_test", "")
                .replace("test_", "")
                .replace(".test", "")
                .replace(".spec", "");
            let source = sources.get(&stem)?;
            Some(GraphRelationRecord {
                id: format!("relation:{}:test_of:{}", test.path, source.path),
                from_type: "file".to_string(),
                from_id: test.id.clone(),
                to_type: "file".to_string(),
                to_id: source.id.clone(),
                relation_kind: "test_of".to_string(),
                confidence: "medium".to_string(),
                source: "filename-heuristic".to_string(),
            })
        })
        .collect()
}

fn build_config_relations(files: &[GraphFileRecord]) -> Vec<GraphRelationRecord> {
    files
        .iter()
        .filter(|file| file.is_config)
        .map(|file| GraphRelationRecord {
            id: format!("relation:{}:configures:project", file.path),
            from_type: "file".to_string(),
            from_id: file.id.clone(),
            to_type: "project".to_string(),
            to_id: "project".to_string(),
            relation_kind: "configures".to_string(),
            confidence: "medium".to_string(),
            source: "config-classifier".to_string(),
        })
        .collect()
}

fn build_same_directory_relations(files: &[GraphFileRecord]) -> Vec<GraphRelationRecord> {
    let mut relations = Vec::new();
    let mut by_dir: BTreeMap<String, Vec<&GraphFileRecord>> = BTreeMap::new();
    for file in files {
        let directory = file
            .path
            .rsplit_once('/')
            .map(|value| value.0)
            .unwrap_or("");
        by_dir.entry(directory.to_string()).or_default().push(file);
    }
    let mut seen = BTreeSet::new();
    for group in by_dir.values() {
        for file in group.iter().take(8) {
            for other in group.iter().take(8) {
                if file.id == other.id {
                    continue;
                }
                let id = format!("relation:{}:same_directory:{}", file.path, other.path);
                if seen.insert(id.clone()) {
                    relations.push(GraphRelationRecord {
                        id,
                        from_type: "file".to_string(),
                        from_id: file.id.clone(),
                        to_type: "file".to_string(),
                        to_id: other.id.clone(),
                        relation_kind: "same_directory".to_string(),
                        confidence: "low".to_string(),
                        source: "directory-heuristic".to_string(),
                    });
                }
            }
        }
    }
    relations
}

fn source_stem(name: &str) -> String {
    name.split('.').next().unwrap_or(name).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scan_project_skips_runtime_and_detects_core_files() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/output/graph")).unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("tests")).unwrap();
        fs::write(
            dir.path().join("src/lib.rs"),
            "pub struct Lease {}\nuse crate::x;\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("tests/lease_test.rs"),
            "fn lease_smoke() {}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        fs::write(dir.path().join(".agentflow/output/graph/meta.json"), "{}").unwrap();

        let index = scan_project(dir.path()).unwrap();

        assert_eq!(index.files.len(), 3);
        assert!(index.files.iter().any(|file| file.path == "src/lib.rs"));
        assert!(index.symbols.iter().any(|symbol| symbol.name == "Lease"));
        assert!(index
            .relations
            .iter()
            .any(|relation| relation.relation_kind == "imports"));
        assert!(index
            .relations
            .iter()
            .any(|relation| relation.relation_kind == "configures"));
    }
}
