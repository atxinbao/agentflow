use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFilesSnapshot {
    pub(crate) version: String,
    pub(crate) project_root: String,
    pub(crate) entries: Vec<ProjectFileEntry>,
    pub(crate) selected_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileEntry {
    pub(crate) name: String,
    pub(crate) relative_path: String,
    pub(crate) kind: String,
    pub(crate) created_at: Option<u64>,
    pub(crate) modified_at: Option<u64>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) extension: Option<String>,
    pub(crate) child_count: Option<usize>,
    pub(crate) is_symlink: bool,
    pub(crate) children: Vec<ProjectFileChild>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileChild {
    pub(crate) name: String,
    pub(crate) relative_path: String,
    pub(crate) kind: String,
    pub(crate) created_at: Option<u64>,
    pub(crate) modified_at: Option<u64>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) extension: Option<String>,
    pub(crate) child_count: Option<usize>,
    pub(crate) is_symlink: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileContent {
    pub(crate) relative_path: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) created_at: Option<u64>,
    pub(crate) modified_at: Option<u64>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) extension: Option<String>,
    pub(crate) mime_type: Option<String>,
    pub(crate) language: String,
    pub(crate) content: Option<String>,
    pub(crate) binary_preview: Option<String>,
    pub(crate) data_url: Option<String>,
    pub(crate) truncated: bool,
    pub(crate) directory_children: Vec<ProjectFileChild>,
    pub(crate) unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectDirectoryPage {
    pub(crate) version: String,
    pub(crate) project_root: String,
    pub(crate) directory_path: String,
    pub(crate) entries: Vec<ProjectFileChild>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) total_children: usize,
    pub(crate) limit: usize,
    pub(crate) view_mode: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileSearchSnapshot {
    pub(crate) version: String,
    pub(crate) project_root: String,
    pub(crate) query: String,
    pub(crate) view_mode: String,
    pub(crate) results: Vec<ProjectFileSearchResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileSearchResult {
    pub(crate) name: String,
    pub(crate) relative_path: String,
    pub(crate) kind: String,
    pub(crate) extension: Option<String>,
    pub(crate) modified_at: Option<u64>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) score: u32,
    pub(crate) match_reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileTextRange {
    pub(crate) version: String,
    pub(crate) project_root: String,
    pub(crate) relative_path: String,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) total_lines: usize,
    pub(crate) content: String,
    pub(crate) truncated: bool,
}
