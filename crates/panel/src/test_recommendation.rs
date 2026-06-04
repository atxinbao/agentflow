use crate::{manager::load_project_panel_manifest, model::PanelTestHint};
use anyhow::Result;
use std::{fs, path::Path};

pub fn recommend_panel_tests(
    project_root: impl AsRef<Path>,
    changed_files: &[String],
    context_files: &[String],
    affected_tests: &[String],
) -> Result<Vec<PanelTestHint>> {
    let root = project_root.as_ref();
    let manifest = load_project_panel_manifest(root)?;
    let mut hints = Vec::new();

    if root.join("Cargo.toml").is_file()
        || manifest.languages.iter().any(|language| language == "rust")
    {
        hints.push(hint(
            "cargo test",
            "检测到 Rust 项目或 Cargo.toml。",
            "medium",
            "full",
        ));
        if let Some(keyword) = focused_keyword(changed_files, context_files) {
            hints.push(hint(
                &format!("cargo test {keyword}"),
                "根据变更文件名生成 focused Rust 测试提示。",
                "low",
                "focused",
            ));
        }
    }

    if root.join("package.json").is_file()
        || manifest
            .languages
            .iter()
            .any(|language| language == "typescript" || language == "javascript")
    {
        hints.push(hint(
            &node_test_command(root),
            "检测到 Node / TypeScript 项目或 package.json test script。",
            "medium",
            "package",
        ));
    }

    if root.join("pyproject.toml").is_file()
        || root.join("pytest.ini").is_file()
        || manifest
            .languages
            .iter()
            .any(|language| language == "python")
    {
        hints.push(hint(
            "python -m pytest",
            "检测到 Python 项目。",
            "medium",
            "full",
        ));
    }

    if root.join("go.mod").is_file() || manifest.languages.iter().any(|language| language == "go") {
        hints.push(hint("go test ./...", "检测到 Go 项目。", "medium", "full"));
    }

    if root.join("pom.xml").is_file() {
        hints.push(hint("mvn test", "检测到 Maven 项目。", "medium", "full"));
    }
    if root.join("build.gradle").is_file() || root.join("build.gradle.kts").is_file() {
        hints.push(hint(
            "./gradlew test",
            "检测到 Gradle / Android 项目。",
            "medium",
            "full",
        ));
        if manifest
            .platforms
            .iter()
            .any(|platform| platform == "android")
        {
            hints.push(hint(
                "./gradlew connectedAndroidTest",
                "检测到 Android 项目，instrumented test 需要设备或模拟器。",
                "low",
                "module",
            ));
        }
    }

    if root.join("Package.swift").is_file() {
        hints.push(hint(
            "swift test",
            "检测到 Swift Package。",
            "medium",
            "full",
        ));
    }
    if manifest.platforms.iter().any(|platform| platform == "ios")
        || has_extension(root, "xcodeproj")
        || has_extension(root, "xcworkspace")
    {
        hints.push(hint(
            "xcodebuild test",
            "检测到 iOS / Apple 项目入口，具体 scheme 需要由执行器补全。",
            "low",
            "full",
        ));
    }

    if root.join("pubspec.yaml").is_file()
        || manifest
            .platforms
            .iter()
            .any(|platform| platform == "flutter")
    {
        hints.push(hint(
            "flutter test",
            "检测到 Flutter 项目。",
            "medium",
            "full",
        ));
    } else if manifest.languages.iter().any(|language| language == "dart") {
        hints.push(hint("dart test", "检测到 Dart 项目。", "low", "full"));
    }

    if has_extension(root, "csproj")
        || manifest
            .languages
            .iter()
            .any(|language| language == "csharp")
    {
        hints.push(hint(
            "dotnet test",
            "检测到 C# / .NET 项目。",
            "medium",
            "full",
        ));
    }
    if root.join("composer.json").is_file()
        || manifest.languages.iter().any(|language| language == "php")
    {
        hints.push(hint(
            "vendor/bin/phpunit",
            "检测到 PHP 项目。",
            "low",
            "full",
        ));
    }
    if root.join("Gemfile").is_file()
        || manifest.languages.iter().any(|language| language == "ruby")
    {
        hints.push(hint(
            "bundle exec rspec",
            "检测到 Ruby 项目。",
            "low",
            "full",
        ));
    }

    for test in affected_tests.iter().take(5) {
        hints.push(hint(
            &format!("focused test for {test}"),
            "影响分析返回了相关测试文件。",
            "low",
            "focused",
        ));
    }

    dedup_hints(hints)
}

fn hint(command_hint: &str, reason: &str, confidence: &str, scope: &str) -> PanelTestHint {
    PanelTestHint {
        command_hint: command_hint.to_string(),
        reason: reason.to_string(),
        confidence: confidence.to_string(),
        scope: scope.to_string(),
    }
}

fn node_test_command(root: &Path) -> String {
    let package_json = root.join("package.json");
    if let Ok(content) = fs::read_to_string(&package_json) {
        if content.contains("\"test\"") {
            if root.join("pnpm-lock.yaml").is_file() {
                return "pnpm test".to_string();
            }
            if root.join("yarn.lock").is_file() {
                return "yarn test".to_string();
            }
            return "npm test".to_string();
        }
        if content.contains("vitest") {
            return "npx vitest run".to_string();
        }
        if content.contains("jest") {
            return "npx jest".to_string();
        }
    }
    "npm test".to_string()
}

fn focused_keyword(changed_files: &[String], context_files: &[String]) -> Option<String> {
    changed_files
        .iter()
        .chain(context_files.iter())
        .filter_map(|path| {
            Path::new(path)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| {
                    value
                        .replace("_test", "")
                        .replace(".test", "")
                        .replace(".spec", "")
                })
        })
        .find(|value| value.len() > 2)
}

fn has_extension(root: &Path, extension: &str) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(extension))
            .unwrap_or(false)
    })
}

fn dedup_hints(hints: Vec<PanelTestHint>) -> Result<Vec<PanelTestHint>> {
    let mut seen = std::collections::BTreeSet::new();
    Ok(hints
        .into_iter()
        .filter(|hint| seen.insert(hint.command_hint.clone()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::index_project_panel;
    use tempfile::tempdir;

    #[test]
    fn recommends_rust_and_node_test_commands() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("package.json"),
            "{\"scripts\":{\"test\":\"vitest run\"}}",
        )
        .unwrap();
        index_project_panel(dir.path()).unwrap();

        let hints = recommend_panel_tests(dir.path(), &[], &[], &[]).unwrap();

        assert!(hints.iter().any(|hint| hint.command_hint == "cargo test"));
        assert!(hints.iter().any(|hint| hint.command_hint == "npm test"));
        assert!(hints.iter().all(|hint| !hint.scope.is_empty()));
    }
}
