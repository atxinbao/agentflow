use crate::{
    model::{
        RootAgentEntryShadowGuardStatus, WorkspaceLayoutStatus, WorkspaceManifest,
        WorkspaceManifestRootEntries, WorkspaceManifestStatus, WORKSPACE_LAYOUT_VERSION,
        WORKSPACE_MANIFEST_VERSION,
    },
    templates::WORKSPACE_MANIFEST_RELATIVE_PATH,
};
use anyhow::{Context, Result};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) const SHADOW_CANDIDATES: [&str; 8] = [
    ".rules",
    ".cursorrules",
    ".windsurfrules",
    ".clinerules",
    ".github/copilot-instructions.md",
    "AGENT.md",
    "CLAUDE.md",
    "GEMINI.md",
];

const LAYOUT_DIRECTORIES: &[&str] = &[
    ".agentflow/define/agent",
    ".agentflow/define/agent/skills",
    ".agentflow/define/agent/state",
    ".agentflow/define/spec",
    ".agentflow/define/spec/skills",
    ".agentflow/define/spec/templates",
    ".agentflow/define/tdd",
    ".agentflow/define/tdd/skills",
    ".agentflow/define/tdd/templates",
    ".agentflow/define/release",
    ".agentflow/define/release/skills",
    ".agentflow/define/release/templates",
    ".agentflow/define/audit",
    ".agentflow/define/audit/skills",
    ".agentflow/define/audit/templates",
    ".agentflow/spec",
    ".agentflow/spec/changes",
    ".agentflow/spec/approvals",
    ".agentflow/spec/drafts",
    ".agentflow/goal-tree",
    ".agentflow/goal-tree/goals",
    ".agentflow/goal-tree/milestones",
    ".agentflow/goal-tree/issues",
    ".agentflow/goal-tree/materialization",
    ".agentflow/panel",
    ".agentflow/panel/context-packs",
    ".agentflow/panel/search",
    ".agentflow/panel/snapshots",
    ".agentflow/panel/index",
    ".agentflow/execute",
    ".agentflow/execute/runs",
    ".agentflow/execute/leases",
    ".agentflow/execute/commands",
    ".agentflow/output",
    ".agentflow/output/evidence",
    ".agentflow/output/audit",
    ".agentflow/output/backup",
    ".agentflow/output/backup/agent-md",
    ".agentflow/output/logs",
    ".agentflow/output/cache",
    ".agentflow/output/tmp",
    ".agentflow/state",
    ".agentflow/state/health",
    ".agentflow/state/locks",
    ".agentflow/state/sessions",
    ".agentflow/state/indexes",
];

const LAYOUT_FILES: [(&str, &str); 5] = [
    (".agentflow/define/spec/SPEC.md", SPEC_MANUAL),
    (".agentflow/define/tdd/TDD.md", TDD_MANUAL),
    (".agentflow/define/release/RELEASE.md", RELEASE_MANUAL),
    (".agentflow/define/audit/AUDIT.md", AUDIT_MANUAL),
    (".agentflow/spec/index.json", SPEC_INDEX),
];

pub(crate) fn prepare_workspace_layout(
    root: &Path,
    warnings: &[String],
    repairs: &mut Vec<String>,
) -> Result<WorkspaceLayoutStatus> {
    let mut created_paths = Vec::new();
    let mut reused_paths = Vec::new();

    for relative_path in LAYOUT_DIRECTORIES.iter().copied() {
        ensure_directory(
            &root.join(relative_path),
            root,
            &mut created_paths,
            &mut reused_paths,
        )?;
    }

    for (relative_path, content) in LAYOUT_FILES {
        write_file_if_changed(
            &root.join(relative_path),
            content,
            root,
            &mut created_paths,
            &mut reused_paths,
            repairs,
        )?;
    }

    let manifest = expected_workspace_manifest(root, warnings);
    let manifest_path = root.join(WORKSPACE_MANIFEST_RELATIVE_PATH);
    let manifest_content = serde_json::to_string_pretty(&manifest)? + "\n";
    write_file_if_changed(
        &manifest_path,
        &manifest_content,
        root,
        &mut created_paths,
        &mut reused_paths,
        repairs,
    )?;

    repairs.extend(
        created_paths
            .iter()
            .map(|path| format!("Created AgentFlow layout path {path}")),
    );

    Ok(WorkspaceLayoutStatus {
        version: WORKSPACE_LAYOUT_VERSION.to_string(),
        ready: true,
        created_paths,
        reused_paths,
        missing_paths: Vec::new(),
    })
}

pub(crate) fn validate_workspace_layout(
    root: &Path,
) -> Result<(WorkspaceManifestStatus, WorkspaceLayoutStatus)> {
    let missing_paths = LAYOUT_DIRECTORIES
        .iter()
        .copied()
        .chain(LAYOUT_FILES.into_iter().map(|(path, _)| path))
        .filter(|relative_path| !root.join(relative_path).exists())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let manifest_path = root.join(WORKSPACE_MANIFEST_RELATIVE_PATH);
    let manifest = read_workspace_manifest(&manifest_path).ok();
    let manifest_exists = manifest_path.exists();
    let manifest_valid = manifest
        .as_ref()
        .map(|value| {
            value.version == WORKSPACE_MANIFEST_VERSION
                && value.layout_version == WORKSPACE_LAYOUT_VERSION
                && value.root_entries.canonical_agent_entry == "AGENTS.md"
        })
        .unwrap_or(false);
    let layout_version = manifest.as_ref().map(|value| value.layout_version.clone());

    Ok((
        WorkspaceManifestStatus {
            exists: manifest_exists,
            path: WORKSPACE_MANIFEST_RELATIVE_PATH.to_string(),
            valid: manifest_valid,
            layout_version,
        },
        WorkspaceLayoutStatus {
            version: WORKSPACE_LAYOUT_VERSION.to_string(),
            ready: missing_paths.is_empty() && manifest_valid,
            created_paths: Vec::new(),
            reused_paths: Vec::new(),
            missing_paths,
        },
    ))
}

pub(crate) fn detect_shadow_files(root: &Path) -> RootAgentEntryShadowGuardStatus {
    let detected = SHADOW_CANDIDATES
        .into_iter()
        .filter(|relative_path| {
            let path = root.join(relative_path);
            if !path.exists() {
                return false;
            }
            fs::read_to_string(&path)
                .map(|content| !content.contains("AGENTFLOW:MANAGED"))
                .unwrap_or(true)
        })
        .map(str::to_string)
        .collect::<Vec<_>>();

    RootAgentEntryShadowGuardStatus {
        checked: SHADOW_CANDIDATES.into_iter().map(str::to_string).collect(),
        detected,
    }
}

pub(crate) fn shadow_warnings(shadow_guard: &RootAgentEntryShadowGuardStatus) -> Vec<String> {
    shadow_guard
        .detected
        .iter()
        .map(|path| {
            format!(
                "Agent entry shadow detected: {path} exists and may be read before AGENTS.md by some tools. AgentFlow uses AGENTS.md as canonical entry."
            )
        })
        .collect()
}

pub(crate) fn expected_workspace_manifest(root: &Path, warnings: &[String]) -> WorkspaceManifest {
    WorkspaceManifest {
        version: WORKSPACE_MANIFEST_VERSION.to_string(),
        layout_version: WORKSPACE_LAYOUT_VERSION.to_string(),
        project_root: root.display().to_string(),
        root_entries: WorkspaceManifestRootEntries {
            canonical_agent_entry: "AGENTS.md".to_string(),
            legacy_agent_entry: "AGENT.MD".to_string(),
            shadow_checked: SHADOW_CANDIDATES.into_iter().map(str::to_string).collect(),
        },
        active_layers: vec![
            "workspace".to_string(),
            "agent-manual".to_string(),
            "panel".to_string(),
            "project-file-reader".to_string(),
            "requirement-intake".to_string(),
        ],
        planned_layers: vec![
            "spec".to_string(),
            "goal-tree".to_string(),
            "tdd".to_string(),
            "execution".to_string(),
            "release".to_string(),
            "audit".to_string(),
        ],
        paths: BTreeMap::from([
            ("agentEntry".to_string(), "AGENTS.md".to_string()),
            ("legacyAgentEntry".to_string(), "AGENT.MD".to_string()),
            (
                "defineAgent".to_string(),
                ".agentflow/define/agent".to_string(),
            ),
            (
                "defineSpec".to_string(),
                ".agentflow/define/spec".to_string(),
            ),
            ("defineTdd".to_string(), ".agentflow/define/tdd".to_string()),
            (
                "defineRelease".to_string(),
                ".agentflow/define/release".to_string(),
            ),
            (
                "defineAudit".to_string(),
                ".agentflow/define/audit".to_string(),
            ),
            ("spec".to_string(), ".agentflow/spec".to_string()),
            ("goalTree".to_string(), ".agentflow/goal-tree".to_string()),
            ("panel".to_string(), ".agentflow/panel".to_string()),
            (
                "panelManifest".to_string(),
                ".agentflow/panel/manifest.json".to_string(),
            ),
            (
                "panelFileTree".to_string(),
                ".agentflow/panel/file-tree.json".to_string(),
            ),
            (
                "panelLanguages".to_string(),
                ".agentflow/panel/languages.json".to_string(),
            ),
            (
                "panelSymbols".to_string(),
                ".agentflow/panel/symbols.json".to_string(),
            ),
            (
                "panelRelations".to_string(),
                ".agentflow/panel/relations.json".to_string(),
            ),
            (
                "panelDiagnostics".to_string(),
                ".agentflow/panel/diagnostics.json".to_string(),
            ),
            (
                "panelGit".to_string(),
                ".agentflow/panel/git.json".to_string(),
            ),
            (
                "panelTests".to_string(),
                ".agentflow/panel/tests.json".to_string(),
            ),
            (
                "panelSearch".to_string(),
                ".agentflow/panel/search".to_string(),
            ),
            (
                "panelContextPacks".to_string(),
                ".agentflow/panel/context-packs".to_string(),
            ),
            (
                "panelSnapshots".to_string(),
                ".agentflow/panel/snapshots".to_string(),
            ),
            (
                "panelIndex".to_string(),
                ".agentflow/panel/index".to_string(),
            ),
            ("execute".to_string(), ".agentflow/execute".to_string()),
            ("output".to_string(), ".agentflow/output".to_string()),
            ("state".to_string(), ".agentflow/state".to_string()),
        ]),
        compat: BTreeMap::from([
            (
                "legacyGoalTreeDefine".to_string(),
                ".agentflow/define".to_string(),
            ),
            ("legacyAgentEntry".to_string(), "AGENT.MD".to_string()),
            (
                "agentflowSkills".to_string(),
                ".agentflow/define/agent/skills".to_string(),
            ),
            ("zedProjectSkills".to_string(), ".agents/skills".to_string()),
            ("skillsExport".to_string(), "planned".to_string()),
        ]),
        warnings: warnings.to_vec(),
    }
}

fn read_workspace_manifest(path: &Path) -> Result<WorkspaceManifest> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn ensure_directory(
    path: &Path,
    root: &Path,
    created_paths: &mut Vec<String>,
    reused_paths: &mut Vec<String>,
) -> Result<()> {
    if path.exists() {
        reused_paths.push(relative_or_display(root, path));
        return Ok(());
    }
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
    created_paths.push(relative_or_display(root, path));
    Ok(())
}

fn write_file_if_changed(
    path: &Path,
    content: &str,
    root: &Path,
    created_paths: &mut Vec<String>,
    reused_paths: &mut Vec<String>,
    repairs: &mut Vec<String>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if fs::read_to_string(path).ok().as_deref() == Some(content) {
        reused_paths.push(relative_or_display(root, path));
        return Ok(());
    }
    let existed = path.exists();
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    let relative_path = relative_or_display(root, path);
    if existed {
        repairs.push(format!("Rewrote AgentFlow layout file {relative_path}"));
        reused_paths.push(relative_path);
    } else {
        created_paths.push(relative_path);
    }
    Ok(())
}

fn relative_or_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

const SPEC_MANUAL: &str = r#"# SPEC.md

Version: spec-manual.v1

SPEC is the requirement and acceptance manual for AgentFlow.

## Rules

- Requirement Intake Result is the prerequisite for SPEC work.
- Only `ready-for-openspec` may proceed to SPEC Draft Preview.
- Before human confirmation, Agents may discuss a draft preview but must not write SPEC fact sources.
- Approved SPEC is required before Goal Tree materialization.
- Real SPEC artifacts live under `.agentflow/spec/`, not under `define/`.

## V1 Boundary

V1 creates this manual only. It does not create SPEC changes, approvals, or facts.
"#;

const TDD_MANUAL: &str = r#"# TDD.md

Version: tdd-manual.v1

TDD is the test-first working manual for future Build Agent execution.

## Rules

- Quality standards come from SPEC acceptance criteria.
- TDD does not redefine requirement quality.
- Build Agent is currently not authorized yet.
- Future implementation must derive tests from SPEC and Goal Tree issue context before code changes.
- Red-green-refactor evidence must be recorded before release.

## V1 Boundary

V1 creates this manual only. It does not run tests or authorize implementation.
"#;

const RELEASE_MANUAL: &str = r#"# RELEASE.md

Version: release-manual.v1

Release is the delivery working manual for future Release Agent execution.

## Rules

- Release Agent is currently not authorized yet.
- Future release work may cover commit, PR, review, changelog, release note, deploy, rollback, and release evidence.
- Current stage cannot create PRs.
- Current stage cannot deploy.
- Release must wait for Build and Validation evidence.

## V1 Boundary

V1 creates this manual only. It does not perform release work.
"#;

const AUDIT_MANUAL: &str = r#"# AUDIT.md

Version: audit-manual.v1

Audit is the code review and risk review working manual for future Audit Agent execution.

## Rules

- Audit Agent is currently not authorized yet.
- Future audit checks SPEC alignment, boundary compliance, architecture impact, security / permission / path / data-write risk, test coverage, legacy reintroduction, unauthorized execution, unauthorized writes, model calls, and evidence completeness.
- Audit output belongs under `.agentflow/output/audit/`.

## V1 Boundary

V1 creates this manual only. It does not generate audit reports.
"#;

const SPEC_INDEX: &str = "{\n  \"version\": \"agentflow-spec-index.v1\",\n  \"changes\": [],\n  \"approvals\": [],\n  \"drafts\": []\n}\n";
