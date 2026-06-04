use crate::{
    context_pack::build_panel_context_pack,
    impact::analyze_panel_impact,
    manager::{load_project_panel_status, prepare_project_panel, PanelPrepareMode},
    model::{PanelPreflightSnapshot, PanelStatus},
};
use anyhow::Result;
use std::{path::Path, thread, time::Duration};

pub fn panel_preflight(
    project_root: impl AsRef<Path>,
    target_type: &str,
    target_id: Option<&str>,
    title: &str,
    objective: &str,
    acceptance_criteria: &[String],
) -> Result<PanelPreflightSnapshot> {
    let project_root = project_root.as_ref();
    let initial = load_project_panel_status(project_root)?;
    let status = match initial.status {
        PanelStatus::Missing | PanelStatus::Stale => {
            prepare_project_panel(project_root, PanelPrepareMode::Blocking)?
        }
        PanelStatus::Indexing => wait_for_indexing(project_root)?,
        PanelStatus::Ready | PanelStatus::Degraded | PanelStatus::Failed => initial,
    };

    if status.status == PanelStatus::Failed {
        return Ok(PanelPreflightSnapshot {
            version: "panel-preflight.v1".to_string(),
            project_root: project_root.display().to_string(),
            target_type: target_type.to_string(),
            target_id: target_id.map(str::to_string),
            status: "degraded".to_string(),
            ready: false,
            reason: status
                .last_error
                .unwrap_or_else(|| "Panel 构建失败，阻止自动 AgentRun。".to_string()),
            panel_status: PanelStatus::Failed,
            context_pack_path: None,
            recommended_files: Vec::new(),
            recommended_symbols: Vec::new(),
            recommended_tests: Vec::new(),
            impact_hints: Vec::new(),
            test_hints: Vec::new(),
        });
    }

    let pack = build_panel_context_pack(
        project_root,
        target_type,
        target_id,
        title,
        objective,
        acceptance_criteria,
    )?;
    let impact = analyze_panel_impact(
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
            ".agentflow/panel/context-packs/{}.json",
            id.replace('/', "-")
        )
    });

    Ok(PanelPreflightSnapshot {
        version: "panel-preflight.v1".to_string(),
        project_root: project_root.display().to_string(),
        target_type: target_type.to_string(),
        target_id: target_id.map(str::to_string),
        status: "ready".to_string(),
        ready: status.status == PanelStatus::Ready || status.status == PanelStatus::Degraded,
        reason: if status.status == PanelStatus::Degraded {
            "Panel 可用但存在降级原因，AgentRun 需要记录原因。".to_string()
        } else {
            "Panel 已就绪，Context Pack 已生成。".to_string()
        },
        panel_status: status.status,
        context_pack_path,
        recommended_files: pack.recommended_files,
        recommended_symbols: pack.recommended_symbols,
        recommended_tests: pack.recommended_tests,
        impact_hints: impact.possibly_affected_files,
        test_hints: pack.test_hints,
    })
}

fn wait_for_indexing(project_root: &Path) -> Result<crate::model::PanelStatusSnapshot> {
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(250));
        let status = load_project_panel_status(project_root)?;
        if status.status != PanelStatus::Indexing {
            return Ok(status);
        }
    }
    Ok(load_project_panel_status(project_root)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preflight_builds_missing_panel_and_context_pack() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lease.rs"), "pub struct Lease {}\n").unwrap();

        let snapshot = panel_preflight(
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
            .join(".agentflow/panel/context-packs/issue-lease.json")
            .is_file());
    }

    #[test]
    fn preflight_reports_degraded_when_protection_warns() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        fs::write(dir.path().join(".git/info/exclude"), "*.log\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub struct Lease {}\n").unwrap();

        let snapshot = panel_preflight(
            dir.path(),
            "issue",
            Some("issue-degraded"),
            "Lease",
            "准备降级状态上下文",
            &[],
        )
        .unwrap();

        assert!(snapshot.ready);
        assert_eq!(snapshot.panel_status, PanelStatus::Degraded);
        assert_eq!(snapshot.status, "ready");
        assert!(snapshot.reason.contains("降级"));
    }

    #[test]
    fn preflight_reports_failed_panel_as_not_ready() {
        let dir = tempdir().unwrap();
        let panel_dir = dir.path().join(".agentflow/panel");
        fs::create_dir_all(&panel_dir).unwrap();
        fs::create_dir_all(panel_dir.join("index")).unwrap();
        fs::write(panel_dir.join("index/panel.db"), "").unwrap();
        fs::write(
            panel_dir.join("manifest.json"),
            r#"{
  "version": "panel-manifest.v1",
  "status": "failed",
  "projectRoot": "",
  "backend": "panel",
  "lastIndexedAt": 1,
  "activeSnapshotId": null,
  "paths": {
    "database": ".agentflow/panel/index/panel.db",
    "fileTree": ".agentflow/panel/file-tree.json",
    "languages": ".agentflow/panel/languages.json",
    "symbols": ".agentflow/panel/symbols.json",
    "relations": ".agentflow/panel/relations.json",
    "diagnostics": ".agentflow/panel/diagnostics.json",
    "git": ".agentflow/panel/git.json",
    "tests": ".agentflow/panel/tests.json"
  },
  "summary": {
    "files": 0,
    "languages": 0,
    "symbols": 0,
    "relations": 0,
    "diagnostics": 0,
    "tests": 0
  },
  "worktree": {
    "root": "",
    "gitBranch": null,
    "headSha": null,
    "dirty": false
  },
  "watcher": {
    "status": "not_started",
    "backend": "none"
  },
  "degradedReasons": [],
  "warnings": [],
  "errors": ["fixture failure"]
}"#,
        )
        .unwrap();

        let snapshot = panel_preflight(
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
        assert_eq!(snapshot.panel_status, PanelStatus::Failed);
        assert_eq!(snapshot.reason, "fixture failure");
    }
}
