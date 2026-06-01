use crate::model::{
    GraphChunkRecord, GraphFileRecord, GraphIndex, GraphRelationRecord, GraphStatus,
    GraphStatusSnapshot, GraphSymbolRecord,
};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub(crate) fn open_graph_db(path: &Path) -> Result<Connection> {
    let connection = Connection::open(path).with_context(|| format!("open {}", path.display()))?;
    migrate(&connection)?;
    Ok(connection)
}

pub(crate) fn migrate(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            extension TEXT,
            language TEXT NOT NULL,
            kind TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            line_count INTEGER NOT NULL,
            modified_at INTEGER,
            content_hash TEXT NOT NULL,
            is_source INTEGER NOT NULL,
            is_test INTEGER NOT NULL,
            is_doc INTEGER NOT NULL,
            is_config INTEGER NOT NULL,
            is_generated INTEGER NOT NULL,
            is_ignored INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS symbols (
            id TEXT PRIMARY KEY,
            file_id TEXT NOT NULL,
            language TEXT NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            signature TEXT,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            parent_symbol_id TEXT,
            visibility TEXT,
            path TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS relations (
            id TEXT PRIMARY KEY,
            from_type TEXT NOT NULL,
            from_id TEXT NOT NULL,
            to_type TEXT NOT NULL,
            to_id TEXT NOT NULL,
            relation_kind TEXT NOT NULL,
            confidence TEXT NOT NULL,
            source TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            file_id TEXT NOT NULL,
            symbol_id TEXT,
            path TEXT NOT NULL,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            text TEXT NOT NULL,
            token_estimate INTEGER NOT NULL,
            content_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS context_packs (
            id TEXT PRIMARY KEY,
            target_type TEXT NOT NULL,
            target_id TEXT,
            query TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            graph_revision TEXT,
            recommended_files_json TEXT NOT NULL,
            recommended_symbols_json TEXT NOT NULL,
            recommended_tests_json TEXT NOT NULL,
            impact_hints_json TEXT NOT NULL,
            reason TEXT,
            confidence TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS index_runs (
            id TEXT PRIMARY KEY,
            started_at INTEGER NOT NULL,
            finished_at INTEGER,
            status TEXT NOT NULL,
            project_root TEXT NOT NULL,
            git_head TEXT,
            files_scanned INTEGER NOT NULL DEFAULT 0,
            files_indexed INTEGER NOT NULL DEFAULT 0,
            symbols_indexed INTEGER NOT NULL DEFAULT 0,
            relations_indexed INTEGER NOT NULL DEFAULT 0,
            error TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_files_language ON files(language);
        CREATE INDEX IF NOT EXISTS idx_files_kind ON files(kind);
        CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
        CREATE INDEX IF NOT EXISTS idx_symbols_file_id ON symbols(file_id);
        CREATE INDEX IF NOT EXISTS idx_relations_kind ON relations(relation_kind);
        CREATE INDEX IF NOT EXISTS idx_chunks_file_id ON chunks(file_id);
        "#,
    )?;
    Ok(())
}

pub(crate) fn replace_index(connection: &mut Connection, index: &GraphIndex) -> Result<()> {
    let transaction = connection.transaction()?;
    transaction.execute("DELETE FROM relations", [])?;
    transaction.execute("DELETE FROM chunks", [])?;
    transaction.execute("DELETE FROM symbols", [])?;
    transaction.execute("DELETE FROM files", [])?;

    for file in &index.files {
        insert_file(&transaction, file)?;
    }
    for symbol in &index.symbols {
        insert_symbol(&transaction, symbol)?;
    }
    for relation in &index.relations {
        insert_relation(&transaction, relation)?;
    }
    for chunk in &index.chunks {
        insert_chunk(&transaction, chunk)?;
    }

    transaction.commit()?;
    Ok(())
}

pub(crate) fn insert_index_run_start(
    connection: &Connection,
    id: &str,
    started_at: u64,
    project_root: &str,
    git_head: Option<&str>,
) -> Result<()> {
    connection.execute(
        "INSERT OR REPLACE INTO index_runs (id, started_at, status, project_root, git_head) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, started_at as i64, "indexing", project_root, git_head],
    )?;
    Ok(())
}

pub(crate) fn finish_index_run(
    connection: &Connection,
    id: &str,
    finished_at: u64,
    status: &str,
    index: Option<&GraphIndex>,
    error: Option<&str>,
) -> Result<()> {
    let (files, symbols, relations) = index
        .map(|index| {
            (
                index.files.len() as i64,
                index.symbols.len() as i64,
                index.relations.len() as i64,
            )
        })
        .unwrap_or((0, 0, 0));
    connection.execute(
        r#"
        UPDATE index_runs
        SET finished_at = ?2,
            status = ?3,
            files_scanned = ?4,
            files_indexed = ?4,
            symbols_indexed = ?5,
            relations_indexed = ?6,
            error = ?7
        WHERE id = ?1
        "#,
        params![
            id,
            finished_at as i64,
            status,
            files,
            symbols,
            relations,
            error
        ],
    )?;
    Ok(())
}

pub(crate) fn counts(
    connection: &Connection,
    project_root: &str,
    status: GraphStatus,
) -> Result<GraphStatusSnapshot> {
    let file_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
    let symbol_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))?;
    let relation_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM relations", [], |row| row.get(0))?;
    Ok(GraphStatusSnapshot {
        version: "graph-status.v1".to_string(),
        project_root: project_root.to_string(),
        status,
        file_count: file_count.max(0) as usize,
        symbol_count: symbol_count.max(0) as usize,
        relation_count: relation_count.max(0) as usize,
        updated_at: None,
        last_error: None,
        watcher_status: None,
        preflight_status: None,
        protection_status: None,
        degraded_reasons: Vec::new(),
    })
}

pub(crate) fn fetch_symbols(connection: &Connection) -> Result<Vec<GraphSymbolRecord>> {
    let mut statement = connection.prepare(
        r#"
        SELECT id, file_id, language, name, kind, signature, start_line, end_line,
               parent_symbol_id, visibility, path
        FROM symbols
        ORDER BY path ASC, start_line ASC
        "#,
    )?;
    let rows = statement.query_map([], |row| {
        Ok(GraphSymbolRecord {
            id: row.get(0)?,
            file_id: row.get(1)?,
            language: row.get(2)?,
            name: row.get(3)?,
            kind: row.get(4)?,
            signature: row.get(5)?,
            start_line: row.get::<_, i64>(6)?.max(1) as usize,
            end_line: row.get::<_, i64>(7)?.max(1) as usize,
            parent_symbol_id: row.get(8)?,
            visibility: row.get(9)?,
            path: row.get(10)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub(crate) fn fetch_relations(connection: &Connection) -> Result<Vec<GraphRelationRecord>> {
    let mut statement = connection.prepare(
        r#"
        SELECT id, from_type, from_id, to_type, to_id, relation_kind, confidence, source
        FROM relations
        ORDER BY relation_kind ASC, id ASC
        "#,
    )?;
    let rows = statement.query_map([], |row| {
        Ok(GraphRelationRecord {
            id: row.get(0)?,
            from_type: row.get(1)?,
            from_id: row.get(2)?,
            to_type: row.get(3)?,
            to_id: row.get(4)?,
            relation_kind: row.get(5)?,
            confidence: row.get(6)?,
            source: row.get(7)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub(crate) fn fetch_files(connection: &Connection) -> Result<Vec<GraphFileRecord>> {
    let mut statement = connection.prepare(
        r#"
        SELECT id, path, name, extension, language, kind, size_bytes, line_count, modified_at,
               content_hash, is_source, is_test, is_doc, is_config, is_generated, is_ignored
        FROM files
        ORDER BY path ASC
        "#,
    )?;
    let rows = statement.query_map([], |row| {
        Ok(GraphFileRecord {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            extension: row.get(3)?,
            language: row.get(4)?,
            kind: row.get(5)?,
            size_bytes: row.get::<_, i64>(6)?.max(0) as u64,
            line_count: row.get::<_, i64>(7)?.max(0) as usize,
            modified_at: row
                .get::<_, Option<i64>>(8)?
                .map(|value| value.max(0) as u64),
            content_hash: row.get(9)?,
            is_source: row.get::<_, i64>(10)? != 0,
            is_test: row.get::<_, i64>(11)? != 0,
            is_doc: row.get::<_, i64>(12)? != 0,
            is_config: row.get::<_, i64>(13)? != 0,
            is_generated: row.get::<_, i64>(14)? != 0,
            is_ignored: row.get::<_, i64>(15)? != 0,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn insert_file(connection: &Connection, file: &GraphFileRecord) -> Result<()> {
    connection.execute(
        r#"
        INSERT INTO files (
            id, path, name, extension, language, kind, size_bytes, line_count, modified_at,
            content_hash, is_source, is_test, is_doc, is_config, is_generated, is_ignored
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        "#,
        params![
            file.id,
            file.path,
            file.name,
            file.extension,
            file.language,
            file.kind,
            file.size_bytes as i64,
            file.line_count as i64,
            file.modified_at.map(|value| value as i64),
            file.content_hash,
            file.is_source as i64,
            file.is_test as i64,
            file.is_doc as i64,
            file.is_config as i64,
            file.is_generated as i64,
            file.is_ignored as i64,
        ],
    )?;
    Ok(())
}

fn insert_symbol(connection: &Connection, symbol: &GraphSymbolRecord) -> Result<()> {
    connection.execute(
        r#"
        INSERT INTO symbols (
            id, file_id, language, name, kind, signature, start_line, end_line,
            parent_symbol_id, visibility, path
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        params![
            symbol.id,
            symbol.file_id,
            symbol.language,
            symbol.name,
            symbol.kind,
            symbol.signature,
            symbol.start_line as i64,
            symbol.end_line as i64,
            symbol.parent_symbol_id,
            symbol.visibility,
            symbol.path
        ],
    )?;
    Ok(())
}

fn insert_relation(connection: &Connection, relation: &GraphRelationRecord) -> Result<()> {
    connection.execute(
        r#"
        INSERT INTO relations (
            id, from_type, from_id, to_type, to_id, relation_kind, confidence, source
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            relation.id,
            relation.from_type,
            relation.from_id,
            relation.to_type,
            relation.to_id,
            relation.relation_kind,
            relation.confidence,
            relation.source
        ],
    )?;
    Ok(())
}

fn insert_chunk(connection: &Connection, chunk: &GraphChunkRecord) -> Result<()> {
    connection.execute(
        r#"
        INSERT INTO chunks (
            id, file_id, symbol_id, path, start_line, end_line, text, token_estimate, content_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            chunk.id,
            chunk.file_id,
            chunk.symbol_id,
            chunk.path,
            chunk.start_line as i64,
            chunk.end_line as i64,
            chunk.text,
            chunk.token_estimate as i64,
            chunk.content_hash
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrate_is_idempotent() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("graph.db");
        let connection = open_graph_db(&db_path).unwrap();
        migrate(&connection).unwrap();
        migrate(&connection).unwrap();
    }
}
