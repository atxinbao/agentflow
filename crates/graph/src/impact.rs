use crate::{
    db,
    manager::graph_db_path,
    model::{GraphContextFile, GraphContextHint, GraphContextSymbol, GraphImpactSnapshot},
    search::search_project_graph,
};
use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn analyze_graph_impact(
    project_root: impl AsRef<Path>,
    changed_files: &[String],
    target_files: &[String],
    target_symbols: &[String],
    query: Option<&str>,
) -> Result<GraphImpactSnapshot> {
    let project_root = project_root.as_ref();
    let connection = db::open_graph_db(&graph_db_path(project_root)?)?;
    let files = db::fetch_files(&connection)?;
    let symbols = db::fetch_symbols(&connection)?;
    let relations = db::fetch_relations(&connection)?;

    let files_by_id = files
        .iter()
        .map(|file| (file.id.clone(), file))
        .collect::<BTreeMap<_, _>>();
    let files_by_path = files
        .iter()
        .map(|file| (file.path.clone(), file))
        .collect::<BTreeMap<_, _>>();
    let symbols_by_id = symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), symbol))
        .collect::<BTreeMap<_, _>>();

    let mut seed_file_ids = BTreeSet::new();
    let mut seed_symbol_ids = BTreeSet::new();
    let mut reasons = Vec::new();

    for path in changed_files.iter().chain(target_files.iter()) {
        if let Some(file) = files_by_path.get(path) {
            seed_file_ids.insert(file.id.clone());
            reasons.push(format!("输入文件 {}", path));
        }
    }
    for symbol_query in target_symbols {
        for symbol in symbols.iter().filter(|symbol| {
            symbol.id == *symbol_query || symbol.name.eq_ignore_ascii_case(symbol_query)
        }) {
            seed_symbol_ids.insert(symbol.id.clone());
            seed_file_ids.insert(symbol.file_id.clone());
            reasons.push(format!("输入符号 {}", symbol.name));
        }
    }
    if let Some(query) = query.filter(|query| !query.trim().is_empty()) {
        let search = search_project_graph(project_root, query, Some(20))?;
        for result in search.results {
            if let Some(file) = files_by_path.get(&result.path) {
                seed_file_ids.insert(file.id.clone());
                reasons.push(format!("查询命中 {}", result.path));
            }
            if result.kind == "symbol" {
                for symbol in symbols
                    .iter()
                    .filter(|symbol| symbol.path == result.path && symbol.name == result.title)
                {
                    seed_symbol_ids.insert(symbol.id.clone());
                }
            }
        }
    }

    let mut affected_file_ids = seed_file_ids.clone();
    let mut affected_symbol_ids = seed_symbol_ids.clone();
    let mut test_paths = BTreeSet::new();

    for relation in &relations {
        let from_seed = seed_file_ids.contains(&relation.from_id)
            || seed_symbol_ids.contains(&relation.from_id);
        let to_seed =
            seed_file_ids.contains(&relation.to_id) || seed_symbol_ids.contains(&relation.to_id);
        if !from_seed && !to_seed {
            continue;
        }
        for id in [&relation.from_id, &relation.to_id] {
            if files_by_id.contains_key(id) {
                affected_file_ids.insert(id.clone());
            }
            if symbols_by_id.contains_key(id) {
                affected_symbol_ids.insert(id.clone());
                if let Some(symbol) = symbols_by_id.get(id) {
                    affected_file_ids.insert(symbol.file_id.clone());
                }
            }
        }
        reasons.push(format!(
            "{} 关系命中 {}",
            relation.relation_kind, relation.source
        ));
    }

    for file_id in &affected_file_ids {
        if let Some(file) = files_by_id.get(file_id) {
            if file.is_test {
                test_paths.insert(file.path.clone());
            }
        }
    }
    for relation in relations
        .iter()
        .filter(|relation| relation.relation_kind == "test_of")
    {
        if affected_file_ids.contains(&relation.to_id) {
            if let Some(file) = files_by_id.get(&relation.from_id) {
                test_paths.insert(file.path.clone());
            }
        }
    }

    let possibly_affected_files = affected_file_ids
        .into_iter()
        .filter_map(|id| files_by_id.get(&id))
        .map(|file| GraphContextHint {
            path: file.path.clone(),
            reason: "由文件、符号或关系启发式推导。".to_string(),
            confidence: if file.is_test { "medium" } else { "low" }.to_string(),
        })
        .take(80)
        .collect::<Vec<_>>();

    let possibly_affected_symbols = affected_symbol_ids
        .into_iter()
        .filter_map(|id| symbols_by_id.get(&id))
        .map(|symbol| GraphContextSymbol {
            name: symbol.name.clone(),
            kind: symbol.kind.clone(),
            path: symbol.path.clone(),
            line: symbol.start_line,
            score: 0.72,
        })
        .take(80)
        .collect::<Vec<_>>();

    let possibly_affected_tests = test_paths
        .into_iter()
        .map(|path| GraphContextFile {
            path,
            reason: "测试文件与受影响文件存在 test_of 或路径关系。".to_string(),
            score: 0.78,
        })
        .take(40)
        .collect::<Vec<_>>();

    reasons.sort();
    reasons.dedup();
    Ok(GraphImpactSnapshot {
        version: "panel-impact.v1".to_string(),
        project_root: project_root.display().to_string(),
        possibly_affected_files,
        possibly_affected_symbols,
        possibly_affected_tests,
        confidence: if reasons.is_empty() { "low" } else { "medium" }.to_string(),
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::index_project_graph;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn impact_returns_related_files_and_tests() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("tests")).unwrap();
        fs::write(dir.path().join("src/lease.rs"), "pub struct Lease {}\n").unwrap();
        fs::write(
            dir.path().join("tests/lease_test.rs"),
            "fn lease_test() {}\n",
        )
        .unwrap();
        index_project_graph(dir.path()).unwrap();

        let impact = analyze_graph_impact(
            dir.path(),
            &["src/lease.rs".to_string()],
            &[],
            &["Lease".to_string()],
            None,
        )
        .unwrap();

        assert!(impact
            .possibly_affected_files
            .iter()
            .any(|item| item.path == "src/lease.rs"));
        assert!(impact
            .possibly_affected_tests
            .iter()
            .any(|item| item.path == "tests/lease_test.rs"));
    }
}
