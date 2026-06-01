use crate::{
    context_pack::build_context_pack,
    impact::analyze_graph_impact,
    manager::{load_project_graph_status, prepare_project_graph, GraphPrepareMode},
    model::{GraphPreflightSnapshot, GraphStatus},
};
use anyhow::Result;
use std::{path::Path, thread, time::Duration};

pub fn preflight_graph_for_target(
    project_root: impl AsRef<Path>,
    target_type: &str,
    target_id: Option<&str>,
    title: &str,
    objective: &str,
    acceptance_criteria: &[String],
) -> Result<GraphPreflightSnapshot> {
    let project_root = project_root.as_ref();
    let initial = load_project_graph_status(project_root)?;
    let status = match initial.status {
        GraphStatus::Missing | GraphStatus::Stale => {
            prepare_project_graph(project_root, GraphPrepareMode::Blocking)?
        }
        GraphStatus::Indexing => wait_for_indexing(project_root)?,
        GraphStatus::Ready | GraphStatus::Degraded | GraphStatus::Failed => initial,
    };

    if status.status == GraphStatus::Failed {
        return Ok(GraphPreflightSnapshot {
            version: "graph-preflight.v1".to_string(),
            project_root: project_root.display().to_string(),
            target_type: target_type.to_string(),
            target_id: target_id.map(str::to_string),
            status: "degraded".to_string(),
            ready: false,
            reason: status
                .last_error
                .unwrap_or_else(|| "Graph 构建失败，阻止自动 AgentRun。".to_string()),
            graph_status: GraphStatus::Failed,
            context_pack_path: None,
            recommended_files: Vec::new(),
            recommended_symbols: Vec::new(),
            recommended_tests: Vec::new(),
            impact_hints: Vec::new(),
            test_hints: Vec::new(),
        });
    }

    let pack = build_context_pack(
        project_root,
        target_type,
        target_id,
        title,
        objective,
        acceptance_criteria,
    )?;
    let impact = analyze_graph_impact(
        project_root,
        &[],
        &pack
            .recommended_files
            .iter()
            .map(|file| file.path.clone())
            .collect::<Vec<_>>(),
        &pack
            .recommended_symbols
            .iter()
            .map(|symbol| symbol.name.clone())
            .collect::<Vec<_>>(),
        Some(&pack.query),
    )?;
    let context_pack_path = target_id.map(|id| {
        format!(
            ".agentflow/output/graph/context-packs/{}.json",
            id.replace('/', "-")
        )
    });

    Ok(GraphPreflightSnapshot {
        version: "graph-preflight.v1".to_string(),
        project_root: project_root.display().to_string(),
        target_type: target_type.to_string(),
        target_id: target_id.map(str::to_string),
        status: "ready".to_string(),
        ready: status.status == GraphStatus::Ready || status.status == GraphStatus::Degraded,
        reason: if status.status == GraphStatus::Degraded {
            "Graph 可用但存在降级原因，AgentRun 需要记录原因。".to_string()
        } else {
            "Graph 已就绪，Context Pack 已生成。".to_string()
        },
        graph_status: status.status,
        context_pack_path,
        recommended_files: pack.recommended_files,
        recommended_symbols: pack.recommended_symbols,
        recommended_tests: pack.recommended_tests,
        impact_hints: impact.possibly_affected_files,
        test_hints: pack.test_hints,
    })
}

fn wait_for_indexing(project_root: &Path) -> Result<crate::model::GraphStatusSnapshot> {
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(250));
        let status = load_project_graph_status(project_root)?;
        if status.status != GraphStatus::Indexing {
            return Ok(status);
        }
    }
    Ok(load_project_graph_status(project_root)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preflight_builds_missing_graph_and_context_pack() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lease.rs"), "pub struct Lease {}\n").unwrap();

        let snapshot = preflight_graph_for_target(
            dir.path(),
            "issue",
            Some("issue-lease"),
            "Lease",
            "准备 Lease 上下文",
            &[],
        )
        .unwrap();

        assert!(snapshot.ready);
        assert!(dir
            .path()
            .join(".agentflow/output/graph/context-packs/issue-lease.json")
            .is_file());
    }

    #[test]
    fn preflight_reports_degraded_when_protection_warns() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        fs::write(dir.path().join(".git/info/exclude"), "*.log\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub struct Lease {}\n").unwrap();

        let snapshot = preflight_graph_for_target(
            dir.path(),
            "issue",
            Some("issue-degraded"),
            "Lease",
            "准备降级状态上下文",
            &[],
        )
        .unwrap();

        assert!(snapshot.ready);
        assert_eq!(snapshot.graph_status, GraphStatus::Degraded);
        assert_eq!(snapshot.status, "ready");
        assert!(snapshot.reason.contains("降级"));
    }

    #[test]
    fn preflight_reports_failed_graph_as_not_ready() {
        let dir = tempdir().unwrap();
        let graph_dir = dir.path().join(".agentflow/output/graph");
        fs::create_dir_all(&graph_dir).unwrap();
        fs::write(graph_dir.join("graph.db"), "").unwrap();
        fs::write(
            graph_dir.join("meta.json"),
            r#"{
  "version": "graph.v1",
  "status": "failed",
  "projectRoot": "",
  "graphDb": ".agentflow/output/graph/graph.db",
  "updatedAt": 1,
  "gitHead": null,
  "fileCount": 0,
  "symbolCount": 0,
  "relationCount": 0,
  "lastIndexRunId": null,
  "languages": [],
  "lastError": "fixture failure",
  "degradedReasons": []
}"#,
        )
        .unwrap();

        let snapshot = preflight_graph_for_target(
            dir.path(),
            "issue",
            Some("issue-failed"),
            "Failed",
            "验证失败态",
            &[],
        )
        .unwrap();

        assert!(!snapshot.ready);
        assert_eq!(snapshot.status, "degraded");
        assert_eq!(snapshot.graph_status, GraphStatus::Failed);
        assert_eq!(snapshot.reason, "fixture failure");
    }
}
