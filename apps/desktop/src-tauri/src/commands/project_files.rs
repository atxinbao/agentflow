//! Project File Reader command wrappers.
//!
//! Tauri command names stay stable while the implementation is isolated under
//! `crate::project_files`.

#[tauri::command]
pub(crate) fn load_project_files_snapshot(
    project_root: Option<String>,
    view_mode: Option<String>,
) -> Result<crate::project_files::ProjectFilesSnapshot, String> {
    crate::project_files::load_project_files_snapshot(project_root, view_mode)
}

#[tauri::command]
pub(crate) fn load_project_file_content(
    relative_path: String,
    project_root: Option<String>,
) -> Result<crate::project_files::ProjectFileContent, String> {
    crate::project_files::load_project_file_content(relative_path, project_root)
}

#[tauri::command]
pub(crate) fn load_project_directory_page(
    directory_path: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
    view_mode: Option<String>,
    project_root: Option<String>,
) -> Result<crate::project_files::ProjectDirectoryPage, String> {
    crate::project_files::load_project_directory_page(
        directory_path,
        cursor,
        limit,
        view_mode,
        project_root,
    )
}

#[tauri::command]
pub(crate) fn search_project_files(
    query: String,
    view_mode: Option<String>,
    limit: Option<usize>,
    project_root: Option<String>,
) -> Result<crate::project_files::ProjectFileSearchSnapshot, String> {
    crate::project_files::search_project_files(query, view_mode, limit, project_root)
}

#[tauri::command]
pub(crate) fn load_project_file_text_range(
    relative_path: String,
    start_line: Option<usize>,
    line_count: Option<usize>,
    project_root: Option<String>,
) -> Result<crate::project_files::ProjectFileTextRange, String> {
    crate::project_files::load_project_file_text_range(
        relative_path,
        start_line,
        line_count,
        project_root,
    )
}

#[tauri::command]
pub(crate) fn choose_existing_project_folder() -> Result<Option<String>, String> {
    crate::project_files::choose_existing_project_folder()
}
