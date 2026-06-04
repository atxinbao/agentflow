use crate::{
    db,
    manager::{context_pack_dir, panel_db_path, unix_timestamp_seconds},
    model::{
        PanelContextFile, PanelContextHint, PanelContextPack, PanelContextSymbol,
        PanelSearchResult, PanelTestHint,
    },
    search::search_project_panel,
    test_recommendation::recommend_panel_tests,
};
use anyhow::{Context, Result};
use rusqlite::params;
use std::{fs, path::Path};

pub fn build_panel_context_pack(
    project_root: impl AsRef<Path>,
    target_type: &str,
    target_id: Option<&str>,
    title: &str,
    objective: &str,
    acceptance_criteria: &[String],
) -> Result<PanelContextPack> {
    let query = [title, objective, &acceptance_criteria.join(" ")]
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let search = search_project_panel(&project_root, &query, Some(30))?;
    let recommended_files = recommended_files(&search.results);
    let recommended_symbols = recommended_symbols(&search.results);
    let recommended_tests = recommended_tests(project_root.as_ref(), &query)?;
    let impact_hints = impact_hints(project_root.as_ref(), &recommended_files)?;
    let test_hints = test_hints(project_root.as_ref(), &recommended_tests)?;

    let pack = PanelContextPack {
        version: "panel-context-pack.v1".to_string(),
        target_type: target_type.to_string(),
        target_id: target_id.map(str::to_string),
        query,
        created_at: unix_timestamp_seconds(),
        panel_revision: latest_panel_revision(project_root.as_ref()).ok().flatten(),
        recommended_files,
        recommended_symbols,
        recommended_tests,
        impact_hints,
        test_hints,
        confidence: "medium".to_string(),
    };

    persist_context_pack(project_root, &pack)?;
    Ok(pack)
}

pub fn load_panel_context_pack(
    project_root: impl AsRef<Path>,
    target_id: &str,
) -> Result<Option<PanelContextPack>> {
    let path = context_pack_dir(project_root)?.join(format!("{target_id}.json"));
    if !path.is_file() {
        return Ok(None);
    }
    let pack = serde_json::from_str(&fs::read_to_string(&path)?)?;
    Ok(Some(pack))
}

fn recommended_files(results: &[PanelSearchResult]) -> Vec<PanelContextFile> {
    results
        .iter()
        .filter(|item| item.kind == "file" || item.kind == "chunk")
        .map(|item| PanelContextFile {
            path: item.path.clone(),
            reason: if item.kind == "chunk" {
                "chunk text matches target query".to_string()
            } else {
                "file path or name matches target query".to_string()
            },
            score: item.score,
        })
        .fold(Vec::new(), |mut acc, item| {
            if !acc
                .iter()
                .any(|existing: &PanelContextFile| existing.path == item.path)
            {
                acc.push(item);
            }
            acc
        })
        .into_iter()
        .take(10)
        .collect()
}

fn recommended_symbols(results: &[PanelSearchResult]) -> Vec<PanelContextSymbol> {
    results
        .iter()
        .filter(|item| item.kind == "symbol")
        .map(|item| PanelContextSymbol {
            name: item.title.clone(),
            kind: item
                .symbol_kind
                .clone()
                .unwrap_or_else(|| "symbol".to_string()),
            path: item.path.clone(),
            line: item.line.unwrap_or(1),
            score: item.score,
        })
        .take(12)
        .collect()
}

fn recommended_tests(project_root: &Path, query: &str) -> Result<Vec<PanelContextFile>> {
    let db_path = panel_db_path(project_root)?;
    let connection = db::open_panel_db(&db_path)?;
    let like = format!("%{}%", query.to_ascii_lowercase());
    let mut statement = connection.prepare(
        r#"
        SELECT path
        FROM files
        WHERE is_test = 1 AND (lower(path) LIKE ?1 OR lower(name) LIKE ?1)
        ORDER BY path ASC
        LIMIT 8
        "#,
    )?;
    let rows = statement.query_map(params![like], |row| row.get::<_, String>(0))?;
    let paths = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(paths
        .into_iter()
        .map(|path| PanelContextFile {
            path,
            reason: "test file matches target query".to_string(),
            score: 0.72,
        })
        .collect())
}

fn impact_hints(project_root: &Path, files: &[PanelContextFile]) -> Result<Vec<PanelContextHint>> {
    let db_path = panel_db_path(project_root)?;
    let connection = db::open_panel_db(&db_path)?;
    let mut hints = Vec::new();
    for file in files.iter().take(5) {
        let mut statement = connection.prepare(
            r#"
            SELECT DISTINCT f.path
            FROM relations r
            JOIN files f ON r.from_id = f.id
            JOIN files target ON r.to_id = target.id
            WHERE target.path = ?1 OR r.to_id = ?1
            LIMIT 5
            "#,
        )?;
        let rows = statement.query_map(params![file.path], |row| row.get::<_, String>(0))?;
        for path in rows.collect::<rusqlite::Result<Vec<_>>>()? {
            hints.push(PanelContextHint {
                path,
                reason: "related through panel relation".to_string(),
                confidence: "medium".to_string(),
            });
        }
    }
    hints.sort_by(|left, right| left.path.cmp(&right.path));
    hints.dedup_by(|left, right| left.path == right.path);
    Ok(hints)
}

fn test_hints(
    project_root: &Path,
    recommended_tests: &[PanelContextFile],
) -> Result<Vec<PanelTestHint>> {
    let affected_tests = recommended_tests
        .iter()
        .map(|test| test.path.clone())
        .collect::<Vec<_>>();
    recommend_panel_tests(project_root, &[], &[], &affected_tests)
}

fn persist_context_pack(project_root: impl AsRef<Path>, pack: &PanelContextPack) -> Result<()> {
    let directory = context_pack_dir(&project_root)?;
    let target_id = pack
        .target_id
        .clone()
        .unwrap_or_else(|| "freeform".to_string())
        .replace('/', "-");
    let path = directory.join(format!("{target_id}.json"));
    fs::write(&path, serde_json::to_string_pretty(pack)?)
        .with_context(|| format!("write {}", path.display()))?;

    let db_path = panel_db_path(project_root)?;
    let connection = db::open_panel_db(&db_path)?;
    connection.execute(
        r#"
        INSERT OR REPLACE INTO context_packs (
            id, target_type, target_id, query, created_at, panel_revision,
            recommended_files_json, recommended_symbols_json, recommended_tests_json,
            impact_hints_json, reason, confidence
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        params![
            target_id,
            pack.target_type,
            pack.target_id,
            pack.query,
            pack.created_at as i64,
            pack.panel_revision,
            serde_json::to_string(&pack.recommended_files)?,
            serde_json::to_string(&pack.recommended_symbols)?,
            serde_json::to_string(&pack.recommended_tests)?,
            serde_json::to_string(&pack.impact_hints)?,
            "generated from panel search",
            pack.confidence
        ],
    )?;
    Ok(())
}

fn latest_panel_revision(project_root: &Path) -> Result<Option<String>> {
    let db_path = panel_db_path(project_root)?;
    let connection = db::open_panel_db(&db_path)?;
    let mut statement = connection.prepare(
        "SELECT id FROM index_runs WHERE status = 'ready' ORDER BY finished_at DESC LIMIT 1",
    )?;
    let mut rows = statement.query([])?;
    Ok(rows.next()?.map(|row| row.get(0)).transpose()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::index_project_panel;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn context_pack_is_written_for_target() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lease.rs"), "pub struct Lease {}\n").unwrap();
        index_project_panel(dir.path()).unwrap();

        let pack = build_panel_context_pack(
            dir.path(),
            "issue",
            Some("issue-lease"),
            "Reject duplicate lease",
            "拒绝重复 lease",
            &[],
        )
        .unwrap();

        assert_eq!(pack.version, "panel-context-pack.v1");
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/issue-lease.json")
            .is_file());
    }
}
