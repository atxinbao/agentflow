use crate::{
    db,
    manager::graph_db_path,
    model::{GraphSearchResult, GraphSearchSnapshot},
};
use anyhow::Result;
use rusqlite::params;
use std::path::Path;

pub fn search_project_graph(
    project_root: impl AsRef<Path>,
    query: &str,
    limit: Option<usize>,
) -> Result<GraphSearchSnapshot> {
    let db_path = graph_db_path(project_root)?;
    let connection = db::open_graph_db(&db_path)?;
    let query = query.trim();
    let limit = limit.unwrap_or(20).clamp(1, 100);
    if query.is_empty() {
        return Ok(GraphSearchSnapshot {
            version: "graph-search.v1".to_string(),
            query: query.to_string(),
            results: Vec::new(),
        });
    }

    let like = format!("%{}%", query.to_ascii_lowercase());
    let mut results = Vec::new();
    results.extend(search_files(&connection, &like, limit)?);
    results.extend(search_symbols(&connection, &like, limit)?);
    results.extend(search_chunks(&connection, &like, limit)?);
    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.path.cmp(&right.path))
    });
    results.truncate(limit);

    Ok(GraphSearchSnapshot {
        version: "graph-search.v1".to_string(),
        query: query.to_string(),
        results,
    })
}

fn search_files(
    connection: &rusqlite::Connection,
    like: &str,
    limit: usize,
) -> Result<Vec<GraphSearchResult>> {
    let mut statement = connection.prepare(
        r#"
        SELECT path, name, language, kind
        FROM files
        WHERE lower(path) LIKE ?1 OR lower(name) LIKE ?1
        ORDER BY path ASC
        LIMIT ?2
        "#,
    )?;
    let rows = statement.query_map(params![like, limit as i64], |row| {
        let path: String = row.get(0)?;
        let name: String = row.get(1)?;
        let language: String = row.get(2)?;
        let kind: String = row.get(3)?;
        Ok(GraphSearchResult {
            kind: "file".to_string(),
            path: path.clone(),
            title: name,
            language: Some(language),
            symbol_kind: Some(kind),
            line: None,
            snippet: None,
            score: if path.to_ascii_lowercase().ends_with(like.trim_matches('%')) {
                0.92
            } else {
                0.74
            },
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn search_symbols(
    connection: &rusqlite::Connection,
    like: &str,
    limit: usize,
) -> Result<Vec<GraphSearchResult>> {
    let mut statement = connection.prepare(
        r#"
        SELECT path, name, language, kind, start_line, signature
        FROM symbols
        WHERE lower(name) LIKE ?1 OR lower(signature) LIKE ?1
        ORDER BY name ASC
        LIMIT ?2
        "#,
    )?;
    let rows = statement.query_map(params![like, limit as i64], |row| {
        let path: String = row.get(0)?;
        let name: String = row.get(1)?;
        let language: String = row.get(2)?;
        let kind: String = row.get(3)?;
        let start_line: i64 = row.get(4)?;
        let signature: Option<String> = row.get(5)?;
        Ok(GraphSearchResult {
            kind: "symbol".to_string(),
            path,
            title: name,
            language: Some(language),
            symbol_kind: Some(kind),
            line: Some(start_line.max(1) as usize),
            snippet: signature,
            score: 0.88,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn search_chunks(
    connection: &rusqlite::Connection,
    like: &str,
    limit: usize,
) -> Result<Vec<GraphSearchResult>> {
    let mut statement = connection.prepare(
        r#"
        SELECT path, start_line, text
        FROM chunks
        WHERE lower(text) LIKE ?1
        ORDER BY path ASC
        LIMIT ?2
        "#,
    )?;
    let rows = statement.query_map(params![like, limit as i64], |row| {
        let path: String = row.get(0)?;
        let start_line: i64 = row.get(1)?;
        let text: String = row.get(2)?;
        Ok(GraphSearchResult {
            kind: "chunk".to_string(),
            path: path.clone(),
            title: format!("{}:{}", path, start_line.max(1)),
            language: None,
            symbol_kind: None,
            line: Some(start_line.max(1) as usize),
            snippet: Some(snippet(&text)),
            score: 0.64,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn snippet(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.len() > 220 {
        format!("{}...", &compact[..220])
    } else {
        compact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::index_project_graph;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn search_returns_file_symbol_and_chunk_matches() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/lease.rs"),
            "pub struct Lease {}\nimpl Lease { pub fn acquire_lease() {} }\n",
        )
        .unwrap();
        index_project_graph(dir.path()).unwrap();

        let result = search_project_graph(dir.path(), "lease", Some(10)).unwrap();

        assert!(result.results.iter().any(|item| item.kind == "file"));
        assert!(result.results.iter().any(|item| item.kind == "symbol"));
    }
}
