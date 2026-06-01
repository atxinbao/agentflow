use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
};

use super::{model::ProjectFileTextRange, path_guard::relative_project_path};

pub(crate) const PROJECT_FILE_TEXT_RANGE_DEFAULT_LINES: usize = 240;
pub(crate) const PROJECT_FILE_TEXT_RANGE_MAX_LINES: usize = 1200;

pub(crate) fn read_project_file_text_range(
    root: &Path,
    path: &Path,
    start_line: Option<usize>,
    line_count: Option<usize>,
) -> Result<ProjectFileTextRange, String> {
    if !path.is_file() {
        return Err("text range target is not a file".to_string());
    }

    let requested_start = start_line.unwrap_or(1).max(1);
    let requested_count = line_count
        .unwrap_or(PROJECT_FILE_TEXT_RANGE_DEFAULT_LINES)
        .clamp(1, PROJECT_FILE_TEXT_RANGE_MAX_LINES);
    let requested_end = requested_start.saturating_add(requested_count - 1);
    let file = fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut content = String::new();
    let mut total_lines = 0usize;
    let mut end_line = 0usize;

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|error| format!("read text range {}: {error}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        total_lines += 1;
        if total_lines >= requested_start && total_lines <= requested_end {
            content.push_str(&line);
            end_line = total_lines;
        }
    }

    Ok(ProjectFileTextRange {
        version: "project-file-text-range.v1".to_string(),
        project_root: root.display().to_string(),
        relative_path: relative_project_path(root, path),
        start_line: requested_start,
        end_line,
        total_lines,
        content,
        truncated: end_line < total_lines,
    })
}
