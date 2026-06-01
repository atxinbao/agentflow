use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

const PROJECT_BINARY_PREVIEW_LIMIT_BYTES: usize = 4096;

pub(crate) fn file_language(name: &str, extension: Option<&str>) -> String {
    match name {
        "Cargo.toml" | "Tauri.toml" => return "toml".to_string(),
        "package.json" | "tsconfig.json" => return "json".to_string(),
        "Dockerfile" | "dockerfile" => return "dockerfile".to_string(),
        "Makefile" | "makefile" => return "makefile".to_string(),
        ".gitignore" | ".env" | ".env.example" => return "config".to_string(),
        _ => {}
    }

    match extension.unwrap_or("").to_lowercase().as_str() {
        "md" | "markdown" => "markdown",
        "json" => "json",
        "jsonc" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "css" => "css",
        "html" => "html",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "dart" => "dart",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "cpp",
        "cs" => "csharp",
        "php" => "php",
        "rb" => "ruby",
        "sql" => "sql",
        "ps1" | "psm1" => "powershell",
        "m" | "mm" => "objective-c",
        "gradle" => "gradle",
        "plist" => "xml",
        "xml" => "xml",
        "dockerfile" => "dockerfile",
        "sh" | "bash" | "zsh" => "shell",
        "txt" => "text",
        _ => "text",
    }
    .to_string()
}

pub(crate) fn file_mime_type(name: &str, extension: Option<&str>) -> Option<String> {
    let normalized_name = name.to_lowercase();
    let mime = match extension.unwrap_or("").to_lowercase().as_str() {
        "md" | "markdown" => "text/markdown",
        "json" | "jsonc" => "application/json",
        "toml" => "application/toml",
        "yaml" | "yml" => "application/yaml",
        "rs" | "ts" | "tsx" | "js" | "jsx" | "css" | "html" | "py" | "go" | "java" | "kt"
        | "kts" | "swift" | "dart" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "cs"
        | "php" | "rb" | "sql" | "ps1" | "psm1" | "m" | "mm" | "gradle" | "plist" | "xml"
        | "sh" | "bash" | "zsh" | "txt" => "text/plain",
        "csv" => "text/csv",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ if normalized_name == ".gitignore" => "text/plain",
        _ if normalized_name == "dockerfile" || normalized_name == "makefile" => "text/plain",
        _ => return None,
    };
    Some(mime.to_string())
}

pub(crate) fn preview_data_url(
    mime_type: Option<&str>,
    bytes: &[u8],
    truncated: bool,
) -> Option<String> {
    let mime_type = mime_type?;
    if truncated {
        return None;
    }
    if !(mime_type.starts_with("image/")
        || mime_type.starts_with("audio/")
        || mime_type.starts_with("video/")
        || mime_type == "application/pdf"
        || mime_type == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        || mime_type == "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
    {
        return None;
    }
    Some(format!(
        "data:{};base64,{}",
        mime_type,
        BASE64_STANDARD.encode(bytes)
    ))
}

pub(crate) fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(PROJECT_BINARY_PREVIEW_LIMIT_BYTES)
        .enumerate()
        .map(|(index, byte)| {
            if index % 16 == 0 {
                format!("\n{:08x}  {:02x}", index, byte)
            } else {
                format!(" {:02x}", byte)
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub(crate) fn binary_unsupported_reason(truncated: bool) -> String {
    if truncated {
        "二进制文件较大，已展示 metadata 和前 4KB 十六进制预览。".to_string()
    } else {
        "二进制文件已展示 metadata 和十六进制预览。".to_string()
    }
}
