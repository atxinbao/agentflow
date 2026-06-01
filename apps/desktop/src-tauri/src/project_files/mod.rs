//! Project File Reader backend.
//!
//! This module owns read-only project file browsing, file content loading,
//! directory pagination, search, and text range loading. It must not write to
//! the workspace.

mod commands;
mod content;
mod directory;
mod mime;
mod model;
mod path_guard;
mod range;
mod search;

pub(crate) use commands::{
    choose_existing_project_folder, load_project_directory_page, load_project_file_content,
    load_project_file_text_range, load_project_files_snapshot, search_project_files,
};
pub(crate) use model::{
    ProjectDirectoryPage, ProjectFileContent, ProjectFileSearchSnapshot, ProjectFileTextRange,
    ProjectFilesSnapshot,
};
