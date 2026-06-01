use std::{fs, io::Read, path::Path};

use super::{
    directory::read_project_file_children,
    mime::{
        binary_unsupported_reason, file_language, file_mime_type, hex_preview, preview_data_url,
    },
    model::ProjectFileContent,
    path_guard::{
        created_at_seconds, modified_at_seconds, project_file_node, relative_project_path,
    },
};

const PROJECT_FILE_PREVIEW_LIMIT_BYTES: u64 = 512 * 1024;

pub(crate) fn read_project_file_content(
    root: &Path,
    path: &Path,
) -> Result<ProjectFileContent, String> {
    let node = project_file_node(root, path)?;
    let metadata = node.metadata;
    let relative_path = relative_project_path(root, path);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_string);
    let language = file_language(&name, extension.as_deref());
    let mime_type = file_mime_type(&name, extension.as_deref());

    if metadata.is_dir() {
        return Ok(ProjectFileContent {
            relative_path,
            name,
            kind: "directory".to_string(),
            created_at: created_at_seconds(&metadata),
            modified_at: modified_at_seconds(&metadata),
            size_bytes: None,
            extension,
            mime_type: Some("inode/directory".to_string()),
            language: "directory".to_string(),
            content: None,
            binary_preview: None,
            data_url: None,
            truncated: false,
            directory_children: read_project_file_children(root, path, "all")?,
            unsupported_reason: None,
        });
    }

    let truncated = metadata.len() > PROJECT_FILE_PREVIEW_LIMIT_BYTES;
    let mut file =
        fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(PROJECT_FILE_PREVIEW_LIMIT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {}: {error}", path.display()))?;

    if bytes.iter().any(|byte| *byte == 0) {
        let binary_preview = hex_preview(&bytes);
        let data_url = preview_data_url(mime_type.as_deref(), &bytes, truncated);
        return Ok(ProjectFileContent {
            relative_path,
            name,
            kind: "file".to_string(),
            created_at: created_at_seconds(&metadata),
            modified_at: modified_at_seconds(&metadata),
            size_bytes: Some(metadata.len()),
            extension,
            mime_type,
            language,
            content: None,
            binary_preview: Some(binary_preview),
            data_url,
            truncated,
            directory_children: Vec::new(),
            unsupported_reason: Some(binary_unsupported_reason(truncated)),
        });
    }

    let content = match String::from_utf8(bytes) {
        Ok(content) => content,
        Err(error) => {
            let bytes = error.into_bytes();
            let binary_preview = hex_preview(&bytes);
            let data_url = preview_data_url(mime_type.as_deref(), &bytes, truncated);
            return Ok(ProjectFileContent {
                relative_path,
                name,
                kind: "file".to_string(),
                created_at: created_at_seconds(&metadata),
                modified_at: modified_at_seconds(&metadata),
                size_bytes: Some(metadata.len()),
                extension,
                mime_type,
                language,
                content: None,
                binary_preview: Some(binary_preview),
                data_url,
                truncated,
                directory_children: Vec::new(),
                unsupported_reason: Some(
                    "文件不是 UTF-8 文本，已展示 metadata 和十六进制预览。".to_string(),
                ),
            });
        }
    };
    Ok(ProjectFileContent {
        relative_path,
        name,
        kind: "file".to_string(),
        created_at: created_at_seconds(&metadata),
        modified_at: modified_at_seconds(&metadata),
        size_bytes: Some(metadata.len()),
        extension,
        mime_type,
        language,
        content: Some(content),
        binary_preview: None,
        data_url: None,
        truncated,
        directory_children: Vec::new(),
        unsupported_reason: if truncated {
            Some("文件较大，当前展示前 512KB 文本预览。".to_string())
        } else {
            None
        },
    })
}
