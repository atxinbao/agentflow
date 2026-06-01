use super::{
    content, directory,
    model::{
        ProjectDirectoryPage, ProjectFileContent, ProjectFileSearchSnapshot, ProjectFileTextRange,
        ProjectFilesSnapshot,
    },
    path_guard, range, search,
};

#[cfg(test)]
use super::{
    content::read_project_file_content, directory::read_project_file_entries, mime::file_language,
    path_guard::sanitize_project_relative_path,
};
#[cfg(test)]
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn load_project_files_snapshot(
    project_root: Option<String>,
    view_mode: Option<String>,
) -> Result<ProjectFilesSnapshot, String> {
    let root = path_guard::resolve_agentflow_project_root(project_root)?;
    let view_mode = directory::normalize_project_file_view_mode(view_mode.as_deref());
    let entries = directory::read_project_file_entries(&root, &root, &view_mode)?;
    let selected_path = directory::preferred_project_file_selection(&entries);

    Ok(ProjectFilesSnapshot {
        version: "project-files.v1".to_string(),
        project_root: root.display().to_string(),
        entries,
        selected_path,
    })
}

pub(crate) fn load_project_file_content(
    relative_path: String,
    project_root: Option<String>,
) -> Result<ProjectFileContent, String> {
    let root = path_guard::resolve_agentflow_project_root(project_root)?;
    let target = path_guard::sanitize_project_relative_path(&root, &relative_path)?;
    content::read_project_file_content(&root, &target)
}

pub(crate) fn load_project_directory_page(
    directory_path: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
    view_mode: Option<String>,
    project_root: Option<String>,
) -> Result<ProjectDirectoryPage, String> {
    let root = path_guard::resolve_agentflow_project_root(project_root)?;
    let view_mode = directory::normalize_project_file_view_mode(view_mode.as_deref());
    let relative_path = directory_path.unwrap_or_default();
    let directory = if relative_path.trim().is_empty() {
        root.clone()
    } else {
        path_guard::sanitize_project_relative_path(&root, &relative_path)?
    };
    if !directory.is_dir() {
        return Err("project directory page target is not a directory".to_string());
    }

    let offset = cursor
        .as_deref()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = limit
        .unwrap_or(directory::DIRECTORY_PAGE_DEFAULT_LIMIT)
        .clamp(1, directory::DIRECTORY_PAGE_MAX_LIMIT);
    let all_entries = directory::read_project_file_child_entries(&root, &directory, &view_mode)?;
    let total_children = all_entries.len();
    let entries = all_entries
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let next_offset = offset + entries.len();
    let next_cursor = if next_offset < total_children {
        Some(next_offset.to_string())
    } else {
        None
    };

    Ok(ProjectDirectoryPage {
        version: "project-directory-page.v1".to_string(),
        project_root: root.display().to_string(),
        directory_path: path_guard::relative_project_path(&root, &directory),
        entries,
        next_cursor,
        total_children,
        limit,
        view_mode,
    })
}

pub(crate) fn search_project_files(
    query: String,
    view_mode: Option<String>,
    limit: Option<usize>,
    project_root: Option<String>,
) -> Result<ProjectFileSearchSnapshot, String> {
    let root = path_guard::resolve_agentflow_project_root(project_root)?;
    let view_mode = directory::normalize_project_file_view_mode(view_mode.as_deref());
    let query = query.trim().to_string();
    let limit = limit
        .unwrap_or(search::PROJECT_FILE_SEARCH_DEFAULT_LIMIT)
        .clamp(1, search::PROJECT_FILE_SEARCH_MAX_LIMIT);
    let results = if query.is_empty() {
        Vec::new()
    } else {
        search::search_project_file_entries(&root, &query, &view_mode, limit)?
    };

    Ok(ProjectFileSearchSnapshot {
        version: "project-file-search.v1".to_string(),
        project_root: root.display().to_string(),
        query,
        view_mode,
        results,
    })
}

pub(crate) fn load_project_file_text_range(
    relative_path: String,
    start_line: Option<usize>,
    line_count: Option<usize>,
    project_root: Option<String>,
) -> Result<ProjectFileTextRange, String> {
    let root = path_guard::resolve_agentflow_project_root(project_root)?;
    let target = path_guard::sanitize_project_relative_path(&root, &relative_path)?;
    range::read_project_file_text_range(&root, &target, start_line, line_count)
}

pub(crate) fn choose_existing_project_folder() -> Result<Option<String>, String> {
    let Some(folder) = rfd::FileDialog::new()
        .set_title("选择现有项目文件夹")
        .pick_folder()
    else {
        return Ok(None);
    };

    let canonical_folder = folder
        .canonicalize()
        .map_err(|error| format!("canonicalize selected project folder: {error}"))?;

    if !canonical_folder.is_dir() {
        return Err("selected project path is not a directory".to_string());
    }

    Ok(Some(canonical_folder.to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("agentflow-project-files-{label}-{nonce}"));
        fs::create_dir_all(&root).expect("create temporary project root");
        root
    }

    fn cleanup_project_root(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_file_content_reads_text_metadata() {
        let root = temp_project_root("text");
        let readme = root.join("README.md");
        fs::write(&readme, "# AgentFlow\n\n只读文件阅读器。").expect("write readme");

        let content = read_project_file_content(&root, &readme).expect("read text project file");

        assert_eq!(content.relative_path, "README.md");
        assert_eq!(content.kind, "file");
        assert_eq!(content.language, "markdown");
        assert_eq!(content.mime_type.as_deref(), Some("text/markdown"));
        assert_eq!(content.data_url, None);
        assert_eq!(content.binary_preview, None);
        assert!(!content.truncated);
        assert!(content.content.expect("text content").contains("AgentFlow"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_directory_content_lists_children() {
        let root = temp_project_root("directory");
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("create docs dir");
        fs::write(docs.join("guide.md"), "# Guide").expect("write guide");

        let content = read_project_file_content(&root, &docs).expect("read directory content");

        assert_eq!(content.relative_path, "docs");
        assert_eq!(content.kind, "directory");
        assert_eq!(content.mime_type.as_deref(), Some("inode/directory"));
        assert_eq!(content.language, "directory");
        assert_eq!(content.directory_children.len(), 1);
        assert_eq!(content.directory_children[0].relative_path, "docs/guide.md");

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_content_returns_binary_hex_fallback() {
        let root = temp_project_root("binary");
        let binary_file = root.join("artifact.bin");
        fs::write(&binary_file, [0_u8, 1, 2, 3, 255]).expect("write binary file");

        let content =
            read_project_file_content(&root, &binary_file).expect("read binary project file");

        assert_eq!(content.content, None);
        assert_eq!(content.data_url, None);
        assert!(content
            .binary_preview
            .expect("hex preview")
            .contains("00000000"));
        assert!(content
            .unsupported_reason
            .expect("binary unsupported reason")
            .contains("二进制文件"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_content_returns_data_urls_for_small_previewable_binary_files() {
        let root = temp_project_root("data-url");
        let cases: [(&str, &[u8], &str); 4] = [
            ("picture.png", b"\x89PNG\r\n\x1a\n\0", "data:image/png;base64,"),
            ("report.pdf", b"%PDF-1.4\n\0", "data:application/pdf;base64,"),
            (
                "sheet.xlsx",
                b"PK\x03\x04\0",
                "data:application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;base64,",
            ),
            (
                "document.docx",
                b"PK\x03\x04\0",
                "data:application/vnd.openxmlformats-officedocument.wordprocessingml.document;base64,",
            ),
        ];

        for (name, bytes, expected_prefix) in cases {
            let path = root.join(name);
            fs::write(&path, bytes).expect("write previewable binary file");
            let content =
                read_project_file_content(&root, &path).expect("read previewable binary file");
            let data_url = content.data_url.expect("preview data url");
            assert!(
                data_url.starts_with(expected_prefix),
                "{name} data URL should start with {expected_prefix}, got {data_url}"
            );
            assert!(content.binary_preview.is_some());
            assert_eq!(content.content, None);
        }

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_content_marks_large_text_as_truncated() {
        let root = temp_project_root("large-text");
        let large_log = root.join("large.log");
        fs::write(&large_log, "AgentFlow large line\n".repeat(40_000))
            .expect("write large text file");

        let content = read_project_file_content(&root, &large_log).expect("read large text file");

        assert!(content.truncated);
        assert!(content
            .content
            .expect("large text preview")
            .contains("AgentFlow large line"));
        assert!(content
            .unsupported_reason
            .expect("large text reason")
            .contains("512KB"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_path_rejects_escape() {
        let root = temp_project_root("path");

        let result = sanitize_project_relative_path(&root, "../outside.txt");

        assert!(result.is_err());

        cleanup_project_root(&root);
    }

    #[test]
    fn project_directory_page_paginates_large_directories() {
        let root = temp_project_root("paging");
        for index in 0..150 {
            fs::write(root.join(format!("file-{index:03}.txt")), "page").expect("write paged file");
        }

        let first_page = load_project_directory_page(
            None,
            None,
            Some(80),
            Some("all".to_string()),
            Some(root.display().to_string()),
        )
        .expect("load first directory page");
        let second_page = load_project_directory_page(
            None,
            first_page.next_cursor.clone(),
            Some(80),
            Some("all".to_string()),
            Some(root.display().to_string()),
        )
        .expect("load second directory page");

        assert_eq!(first_page.entries.len(), 80);
        assert_eq!(first_page.total_children, 150);
        assert!(first_page.next_cursor.is_some());
        assert_eq!(second_page.entries.len(), 70);
        assert_eq!(second_page.next_cursor, None);

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_search_finds_matching_paths() {
        let root = temp_project_root("search");
        fs::create_dir_all(root.join("apps/desktop")).expect("create nested dir");
        fs::write(
            root.join("apps/desktop/App.tsx"),
            "export const App = null;",
        )
        .expect("write app");

        let snapshot = search_project_files(
            "App".to_string(),
            Some("all".to_string()),
            Some(20),
            Some(root.display().to_string()),
        )
        .expect("search project files");

        assert!(snapshot
            .results
            .iter()
            .any(|result| result.relative_path == "apps/desktop/App.tsx"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_search_filters_by_extension_query() {
        let root = temp_project_root("search-extension");
        fs::write(root.join("main.rs"), "fn main() {}").expect("write rust file");
        fs::write(root.join("README.md"), "# AgentFlow").expect("write markdown file");

        let snapshot = search_project_files(
            "ext:rs".to_string(),
            Some("all".to_string()),
            Some(20),
            Some(root.display().to_string()),
        )
        .expect("search by extension");

        assert!(snapshot
            .results
            .iter()
            .any(|result| result.relative_path == "main.rs" && result.match_reason == "extension"));
        assert!(!snapshot
            .results
            .iter()
            .any(|result| result.relative_path == "README.md"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_language_recognizes_mobile_and_backend_configs() {
        let cases = [
            ("AndroidManifest.xml", Some("xml"), "xml"),
            ("Info.plist", Some("plist"), "xml"),
            ("pubspec.yaml", Some("yaml"), "yaml"),
            ("build.gradle", Some("gradle"), "gradle"),
            ("query.sql", Some("sql"), "sql"),
            ("App.swift", Some("swift"), "swift"),
            ("main.dart", Some("dart"), "dart"),
            ("server.py", Some("py"), "python"),
            ("main.go", Some("go"), "go"),
            ("ViewController.m", Some("m"), "objective-c"),
        ];

        for (name, extension, expected) in cases {
            assert_eq!(file_language(name, extension), expected, "{name}");
        }
    }

    #[test]
    fn project_file_text_range_reads_selected_lines() {
        let root = temp_project_root("range");
        let file = root.join("long.txt");
        fs::write(&file, "one\ntwo\nthree\nfour\nfive\n").expect("write ranged file");

        let range = load_project_file_text_range(
            "long.txt".to_string(),
            Some(2),
            Some(2),
            Some(root.display().to_string()),
        )
        .expect("load text range");

        assert_eq!(range.start_line, 2);
        assert_eq!(range.end_line, 3);
        assert_eq!(range.total_lines, 5);
        assert_eq!(range.content, "two\nthree\n");
        assert!(range.truncated);

        cleanup_project_root(&root);
    }

    #[test]
    fn source_view_hides_generated_directories() {
        let root = temp_project_root("source-view");
        fs::create_dir_all(root.join("target")).expect("create target dir");
        fs::write(root.join("README.md"), "# Readme").expect("write readme");

        let entries = read_project_file_entries(&root, &root, "source").expect("read source view");

        assert!(entries.iter().any(|entry| entry.name == "README.md"));
        assert!(!entries.iter().any(|entry| entry.name == "target"));

        cleanup_project_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_inside_root_is_allowed_and_outside_root_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = temp_project_root("symlink");
        let outside = temp_project_root("symlink-outside");
        fs::write(root.join("inside.txt"), "inside").expect("write inside");
        fs::write(outside.join("outside.txt"), "outside").expect("write outside");
        symlink(root.join("inside.txt"), root.join("inside-link.txt"))
            .expect("create inside symlink");
        symlink(outside.join("outside.txt"), root.join("outside-link.txt"))
            .expect("create outside symlink");

        let inside = sanitize_project_relative_path(&root, "inside-link.txt");
        let outside_result = sanitize_project_relative_path(&root, "outside-link.txt");

        assert!(inside.is_ok());
        assert!(outside_result.is_err());

        cleanup_project_root(&root);
        cleanup_project_root(&outside);
    }
}
