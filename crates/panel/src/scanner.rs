use crate::{
    model::{PanelFileRecord, PanelIndex, PanelRelationRecord},
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

pub(crate) fn scan_project(root: &Path) -> Result<PanelIndex> {
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
    relations.extend(build_same_module_relations(&files));
    relations.extend(build_mention_relations(&files, &symbols, &file_text_by_id));

    Ok(PanelIndex {
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

fn build_file_record(root: &Path, path: &Path) -> Result<PanelFileRecord> {
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

    Ok(PanelFileRecord {
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
        || lower_name.ends_with("test.kt")
        || lower_name.ends_with("test.swift")
        || lower_name.ends_with("_test.dart")
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

fn build_test_relations(files: &[PanelFileRecord]) -> Vec<PanelRelationRecord> {
    let sources: BTreeMap<String, &PanelFileRecord> = files
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
            Some(PanelRelationRecord {
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

fn build_config_relations(files: &[PanelFileRecord]) -> Vec<PanelRelationRecord> {
    files
        .iter()
        .filter(|file| file.is_config)
        .map(|file| PanelRelationRecord {
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

fn build_same_directory_relations(files: &[PanelFileRecord]) -> Vec<PanelRelationRecord> {
    let mut relations = Vec::new();
    let mut by_dir: BTreeMap<String, Vec<&PanelFileRecord>> = BTreeMap::new();
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
                    relations.push(PanelRelationRecord {
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

fn build_same_module_relations(files: &[PanelFileRecord]) -> Vec<PanelRelationRecord> {
    let mut relations = Vec::new();
    let mut by_module: BTreeMap<String, Vec<&PanelFileRecord>> = BTreeMap::new();
    for file in files
        .iter()
        .filter(|file| file.is_source || file.is_test || file.is_config)
    {
        let module = module_key(&file.path);
        by_module.entry(module).or_default().push(file);
    }
    let mut seen = BTreeSet::new();
    for group in by_module.values() {
        for file in group.iter().take(12) {
            for other in group.iter().take(12) {
                if file.id == other.id {
                    continue;
                }
                let id = format!("relation:{}:same_module:{}", file.path, other.path);
                if seen.insert(id.clone()) {
                    relations.push(PanelRelationRecord {
                        id,
                        from_type: "file".to_string(),
                        from_id: file.id.clone(),
                        to_type: "file".to_string(),
                        to_id: other.id.clone(),
                        relation_kind: "same_module".to_string(),
                        confidence: "low".to_string(),
                        source: "module-path-heuristic".to_string(),
                    });
                }
            }
        }
    }
    relations
}

fn build_mention_relations(
    files: &[PanelFileRecord],
    symbols: &[crate::model::PanelSymbolRecord],
    file_text_by_id: &BTreeMap<String, String>,
) -> Vec<PanelRelationRecord> {
    let mut relations = Vec::new();
    let mut seen = BTreeSet::new();
    let files_by_id = files
        .iter()
        .map(|file| (file.id.clone(), file))
        .collect::<BTreeMap<_, _>>();

    for (file_id, text) in file_text_by_id {
        let Some(file) = files_by_id.get(file_id) else {
            continue;
        };
        let lower_text = text.to_ascii_lowercase();
        for symbol in symbols
            .iter()
            .filter(|symbol| symbol.name.len() > 2)
            .take(500)
        {
            if symbol.file_id == *file_id {
                continue;
            }
            let name = symbol.name.to_ascii_lowercase();
            if !lower_text.contains(&name) {
                continue;
            }
            for relation_kind in ["mentions", "uses"] {
                if relation_kind == "uses" && !file.is_source {
                    continue;
                }
                let id = format!("relation:{}:{}:{}", file.path, relation_kind, symbol.id);
                if seen.insert(id.clone()) {
                    relations.push(PanelRelationRecord {
                        id,
                        from_type: "file".to_string(),
                        from_id: file.id.clone(),
                        to_type: "symbol".to_string(),
                        to_id: symbol.id.clone(),
                        relation_kind: relation_kind.to_string(),
                        confidence: "low".to_string(),
                        source: "symbol-mention".to_string(),
                    });
                }
            }
        }
    }
    relations
}

fn module_key(path: &str) -> String {
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() >= 3 && matches!(parts[0], "src" | "tests" | "app" | "lib") {
        return parts[1].to_string();
    }
    parts.first().copied().unwrap_or("").to_string()
}

fn source_stem(name: &str) -> String {
    name.split('.').next().unwrap_or(name).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{manager::index_project_panel, search::search_project_panel};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scan_project_skips_runtime_and_detects_core_files() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/panel")).unwrap();
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
        fs::write(dir.path().join(".agentflow/panel/manifest.json"), "{}").unwrap();

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

    #[test]
    fn scan_project_extracts_l1_symbols_relations_and_mobile_semantics() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("android/app/src/main")).unwrap();
        fs::create_dir_all(dir.path().join("ios/App")).unwrap();
        fs::create_dir_all(dir.path().join("lib")).unwrap();
        fs::write(
            dir.path().join("src/app.tsx"),
            "import React from 'react';\nexport interface Props {}\nexport class Screen extends Base implements Runnable {\n  render() { return null; }\n}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("src/main.py"),
            "import os\nclass Worker:\n    def run(self):\n        pass\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("src/lib.go"),
            "package demo\nimport \"fmt\"\ntype Repo struct {}\nfunc (r Repo) Save() {}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("android/app/src/main/AndroidManifest.xml"),
            "<manifest>\n<uses-permission android:name=\"android.permission.INTERNET\"/>\n<activity android:name=\".MainActivity\"/>\n</manifest>\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("ios/App/App.swift"),
            "import SwiftUI\nstruct AppView: View { var body: some View { Text(\"x\") } }\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("lib/main.dart"),
            "import 'package:flutter/widgets.dart';\nclass Home extends StatelessWidget {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("pubspec.yaml"), "name: demo\n").unwrap();

        let index = scan_project(dir.path()).unwrap();

        for expected in [
            "Props", "Screen", "Worker", "run", "Repo", "Save", "AppView", "Home",
        ] {
            assert!(
                index.symbols.iter().any(|symbol| symbol.name == expected),
                "missing symbol {expected}"
            );
        }
        for relation_kind in [
            "contains",
            "imports",
            "parent_of",
            "extends",
            "implements",
            "same_module",
        ] {
            assert!(
                index
                    .relations
                    .iter()
                    .any(|relation| relation.relation_kind == relation_kind),
                "missing relation {relation_kind}"
            );
        }
        assert!(
            index
                .symbols
                .iter()
                .any(|symbol| symbol.name.contains("INTERNET")
                    || symbol.name.contains("MainActivity"))
        );
        assert!(index
            .symbols
            .iter()
            .any(|symbol| symbol.kind == "component"));
    }

    #[test]
    fn l1_fixture_matrix_extracts_symbols_imports_relations_and_chunks() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        let fixtures = [
            (
                "src/app.ts",
                "import React from 'react';\nclass TsParent { child() { return React; } }\n",
                "typescript",
            ),
            (
                "src/app.js",
                "import fs from 'fs';\nclass JsParent { child() { return fs; } }\n",
                "javascript",
            ),
            (
                "src/app.py",
                "import os\nclass PyParent:\n    def child(self):\n        return os.getcwd()\n",
                "python",
            ),
            (
                "src/App.java",
                "package demo;\nimport java.util.List;\nclass JavaParent { void child() {} }\n",
                "java",
            ),
            (
                "src/App.kt",
                "package demo\nimport kotlin.String\nclass KotlinParent { fun child(): String = \"ok\" }\n",
                "kotlin",
            ),
            (
                "src/App.swift",
                "import Foundation\nclass SwiftParent { func child() {} }\n",
                "swift",
            ),
            (
                "src/app.go",
                "package demo\nimport \"fmt\"\ntype GoParent struct {}\nfunc (g GoParent) Child() { fmt.Println(\"ok\") }\n",
                "go",
            ),
            (
                "src/lib.rs",
                "use std::fmt;\npub struct RustParent {}\nimpl RustParent { pub fn child(&self) {} }\n",
                "rust",
            ),
            (
                "src/app.c",
                "#include <stdio.h>\nstruct CParent { int id; };\nvoid c_child() {}\n",
                "c",
            ),
            (
                "src/app.cpp",
                "#include <vector>\nclass CppParent { void child() {} };\n",
                "cpp",
            ),
            (
                "src/App.cs",
                "using System;\nclass CsParent { void Child() {} }\n",
                "csharp",
            ),
            (
                "src/app.dart",
                "import 'dart:io';\nclass DartParent { void child() {} }\n",
                "dart",
            ),
        ];
        for (path, content, _) in fixtures {
            fs::write(dir.path().join(path), content).unwrap();
        }

        let index = scan_project(dir.path()).unwrap();

        for (_, _, language) in fixtures {
            assert!(
                index.files.iter().any(|file| file.language == language),
                "missing language file {language}"
            );
            assert!(
                index
                    .symbols
                    .iter()
                    .any(|symbol| symbol.language == language
                        && symbol.start_line >= 1
                        && symbol.end_line >= symbol.start_line),
                "missing positioned symbol for {language}"
            );
            assert!(
                index.chunks.iter().any(|chunk| {
                    index
                        .files
                        .iter()
                        .any(|file| file.id == chunk.file_id && file.language == language)
                }),
                "missing chunk for {language}"
            );
        }
        for relation_kind in ["contains", "imports", "parent_of"] {
            assert!(
                index
                    .relations
                    .iter()
                    .any(|relation| relation.relation_kind == relation_kind),
                "missing relation {relation_kind}"
            );
        }
    }

    #[test]
    fn l2_l3_fixture_matrix_classifies_configs_docs_and_searchable_chunks() {
        let dir = tempdir().unwrap();
        let fixtures = [
            (
                "index.php",
                "<?php use App\\Repo; class PhpRepo {}\n",
                "php",
            ),
            ("app.rb", "require 'json'\nclass RubyRepo\nend\n", "ruby"),
            ("schema.sql", "CREATE TABLE users (id INTEGER);\n", "sql"),
            (
                "run.sh",
                "source ./env\nfunction run_app() { echo ok; }\n",
                "shell",
            ),
            (
                "script.ps1",
                "function Invoke-App { Write-Output \"ok\" }\n",
                "powershell",
            ),
            (
                "index.html",
                "<html><body><main>hello</main></body></html>\n",
                "html",
            ),
            ("style.css", ".app { color: red; }\n", "css"),
            ("README.md", "# Overview\nPanel docs\n", "markdown"),
            (
                "package.json",
                "{\"scripts\":{\"test\":\"vitest\"}}\n",
                "json",
            ),
            ("config.yaml", "service:\n  name: panel\n", "yaml"),
            (
                "Cargo.toml",
                "[package]\nname = \"panel-fixture\"\n",
                "toml",
            ),
            (
                "AndroidManifest.xml",
                "<manifest><application /></manifest>\n",
                "xml",
            ),
            (
                "Info.plist",
                "<plist><dict><key>CFBundleIdentifier</key><string>x</string></dict></plist>\n",
                "plist",
            ),
            (
                "build.gradle",
                "plugins { id 'com.android.application' }\n",
                "gradle",
            ),
            ("Dockerfile", "FROM rust:latest\n", "dockerfile"),
            ("pyproject.toml", "[project]\nname = \"demo\"\n", "toml"),
            ("go.mod", "module example.com/demo\n", "go"),
            ("pubspec.yaml", "name: demo\nflutter:\n", "yaml"),
            (
                "Package.swift",
                "import PackageDescription\nlet package = Package(name: \"Demo\")\n",
                "swift",
            ),
            ("Podfile", "platform :ios, '17.0'\n", "ruby"),
        ];
        for (path, content, _) in fixtures {
            fs::write(dir.path().join(path), content).unwrap();
        }

        let index = scan_project(dir.path()).unwrap();
        assert_eq!(index.files.len(), fixtures.len());
        for (_, _, language) in fixtures {
            assert!(
                index.files.iter().any(|file| file.language == language),
                "missing language {language}"
            );
        }
        assert!(index.files.iter().any(|file| file.kind == "doc"));
        assert!(index.files.iter().any(|file| file.kind == "config"));
        assert!(index
            .chunks
            .iter()
            .any(|chunk| chunk.text.contains("Overview")));

        let status = index_project_panel(dir.path()).unwrap();
        assert_eq!(status.status, crate::model::PanelStatus::Ready);
        let heading = search_project_panel(dir.path(), "Overview", Some(20)).unwrap();
        assert!(heading.results.iter().any(|item| item.path == "README.md"));
        let plist = search_project_panel(dir.path(), "CFBundleIdentifier", Some(20)).unwrap();
        assert!(
            plist.results.iter().any(|item| item.path == "Info.plist"),
            "missing searchable plist config key"
        );
    }

    #[test]
    fn structured_fallback_parser_covers_non_l1_languages() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("app.php"),
            "<?php use App\\Repo; class Controller {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("README.md"), "# Fallback Heading\n").unwrap();

        let index = scan_project(dir.path()).unwrap();

        assert!(index
            .relations
            .iter()
            .any(|relation| relation.source.contains("structured-parser")));
        assert!(index
            .symbols
            .iter()
            .any(|symbol| symbol.kind == "markdown_heading"));
    }
}
