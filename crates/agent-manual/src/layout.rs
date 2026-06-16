use crate::{
    model::{
        AgentLocaleState, AgentStyleState, RootAgentEntryShadowGuardStatus, WorkspaceLayoutStatus,
        WorkspaceManifest, WorkspaceManifestLocale, WorkspaceManifestOwnership,
        WorkspaceManifestRootEntries, WorkspaceManifestStatus, WorkspaceManifestStyle,
        WorkspaceOwnershipState, WORKSPACE_LAYOUT_VERSION, WORKSPACE_MANAGED_BY,
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
    ".agentflow/spec/projects",
    ".agentflow/spec/issues",
    ".agentflow/spec/archive",
    ".agentflow/spec/requirements",
    ".agentflow/events",
    ".agentflow/projections",
    ".agentflow/projections/tasks",
    ".agentflow/projections/projects",
    ".agentflow/indexes",
    ".agentflow/tasks",
    ".agentflow/panel",
    ".agentflow/panel/context-packs",
    ".agentflow/panel/search",
    ".agentflow/panel/snapshots",
    ".agentflow/panel/index",
    ".agentflow/audit",
    ".agentflow/define/agent/backup",
    ".agentflow/define/agent/backup/agent-md",
    ".agentflow/state",
    ".agentflow/state/health",
    ".agentflow/state/locks",
    ".agentflow/state/sessions",
    ".agentflow/state/indexes",
];

const LAYOUT_FILES: [(&str, &str); 5] = [
    (".agentflow/define/agent/roles.json", AGENT_ROLES_JSON),
    (".agentflow/define/spec/SPEC.md", SPEC_MANUAL),
    (".agentflow/define/tdd/TDD.md", TDD_MANUAL),
    (".agentflow/define/release/RELEASE.md", RELEASE_MANUAL),
    (".agentflow/define/audit/AUDIT.md", AUDIT_MANUAL),
];

const AGENT_ROLES_JSON: &str = r#"{
  "version": "agent-roles.v1",
  "roles": [
    {
      "agentRole": "spec-agent",
      "label": "需求助手",
      "allowedIssueCategories": [],
      "allowedWrites": [
        "docs/requirements/**",
        ".agentflow/spec/projects/**",
        ".agentflow/spec/issues/**"
      ],
      "forbiddenWrites": [
        ".agentflow/tasks/**",
        ".agentflow/audit/**"
      ]
    },
    {
      "agentRole": "build-agent",
      "label": "执行助手",
      "allowedIssueCategories": ["spec"],
      "allowedWrites": [
        ".agentflow/tasks/<issue-id>/runs/**",
        ".agentflow/tasks/<issue-id>/evidence/**",
        ".agentflow/events/**"
      ],
      "forbiddenWrites": [
        ".agentflow/audit/**"
      ]
    },
    {
      "agentRole": "audit-agent",
      "label": "审计助手",
      "allowedIssueCategories": ["audit"],
      "allowedWrites": [
        ".agentflow/audit/**",
        ".agentflow/events/**"
      ],
      "forbiddenWrites": [
        ".agentflow/tasks/**"
      ]
    }
  ]
}
"#;

pub(crate) fn prepare_workspace_layout(
    root: &Path,
    warnings: &[String],
    repairs: &mut Vec<String>,
    locale: &AgentLocaleState,
    style: &AgentStyleState,
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

    let manifest = expected_workspace_manifest(root, warnings, locale, style);
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
                && value.managed_by == WORKSPACE_MANAGED_BY
                && value.layout_version == WORKSPACE_LAYOUT_VERSION
                && value.ownership.status == WorkspaceOwnershipState::ManagedCurrent
                && value.root_entries.canonical_agent_entry == "AGENTS.md"
                && value.locale.manual_language == crate::model::MANUAL_LANGUAGE
                && !value.locale.agent_locale.is_empty()
                && value.style.style_id == crate::model::PLAIN_WORK_STYLE_ID
                && value.style.manual_language == crate::model::MANUAL_LANGUAGE
                && value.style.applies_to_code_comments
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

pub(crate) fn expected_workspace_manifest(
    root: &Path,
    warnings: &[String],
    locale: &AgentLocaleState,
    style: &AgentStyleState,
) -> WorkspaceManifest {
    let now = unix_timestamp_seconds();
    WorkspaceManifest {
        version: WORKSPACE_MANIFEST_VERSION.to_string(),
        managed_by: WORKSPACE_MANAGED_BY.to_string(),
        layout_version: WORKSPACE_LAYOUT_VERSION.to_string(),
        project_root: root.display().to_string(),
        ownership: WorkspaceManifestOwnership {
            status: WorkspaceOwnershipState::ManagedCurrent,
            created_by: WORKSPACE_MANAGED_BY.to_string(),
            created_at: now,
            last_validated_at: now,
            migrated_from: None,
            migration_record: None,
        },
        root_entries: WorkspaceManifestRootEntries {
            canonical_agent_entry: "AGENTS.md".to_string(),
            shadow_checked: SHADOW_CANDIDATES.into_iter().map(str::to_string).collect(),
        },
        locale: WorkspaceManifestLocale {
            agent_locale: locale.agent_locale.clone(),
            manual_language: locale.manual_language.clone(),
            raw_os_locale: locale.raw_os_locale.clone(),
            source: locale.source.clone(),
            checked_at: locale.checked_at,
            fallback: locale.fallback,
            warnings: locale.warnings.clone(),
        },
        style: WorkspaceManifestStyle {
            style_id: style.style_id.clone(),
            manual_language: style.manual_language.clone(),
            applies_to_agent_locale: style.applies_to_agent_locale,
            applies_to_code_comments: style.applies_to_code_comments,
        },
        active_layers: vec![
            "workspace".to_string(),
            "agent-manual".to_string(),
            "panel".to_string(),
            "spec".to_string(),
            "events".to_string(),
            "projections".to_string(),
            "tasks".to_string(),
            "project-file-reader".to_string(),
            "requirement-intake".to_string(),
        ],
        planned_layers: vec![
            "tdd".to_string(),
            "execution".to_string(),
            "release".to_string(),
            "audit".to_string(),
        ],
        paths: BTreeMap::from([
            ("agentEntry".to_string(), "AGENTS.md".to_string()),
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
            (
                "specProjects".to_string(),
                ".agentflow/spec/projects".to_string(),
            ),
            (
                "specIssues".to_string(),
                ".agentflow/spec/issues".to_string(),
            ),
            ("events".to_string(), ".agentflow/events".to_string()),
            (
                "projections".to_string(),
                ".agentflow/projections".to_string(),
            ),
            (
                "taskProjections".to_string(),
                ".agentflow/projections/tasks".to_string(),
            ),
            (
                "projectProjections".to_string(),
                ".agentflow/projections/projects".to_string(),
            ),
            ("tasks".to_string(), ".agentflow/tasks".to_string()),
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
            ("state".to_string(), ".agentflow/state".to_string()),
        ]),
        compat: BTreeMap::from([
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

fn unix_timestamp_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
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
- Only `ready-for-spec` may proceed to SPEC Draft Preview.
- Before human confirmation, Agents may discuss a draft preview but must not write SPEC fact sources.
- Public requirement records are required before spec issue generation.
- Requirement records live under `docs/requirements/**`; internal contracts live under `.agentflow/spec/**`, not under `define/`.

## Human-facing Output

- Requirement Intake Result must be structured prose in the user's `agentLocale`.
- SPEC Draft Preview must be structured prose in the user's `agentLocale`.
- Public requirement prose must follow the user's `agentLocale`.
- Spec Project and Issue human-facing fields must follow the user's `agentLocale`.
- Do not show raw JSON as the default conversation output.
- Raw JSON is allowed only for internal records, persisted fact files, tests, or advanced details.

## Public Requirement Artifacts

- `docs/requirements/<requirement-id>.md` is the primary human-readable requirement and acceptance document.
- `.agentflow/spec/projects/**` and `.agentflow/spec/issues/**` store internal task contracts.

## V1 Boundary

V1 creates this manual only. It does not create SPEC changes, approvals, or facts.
"#;

const TDD_MANUAL: &str = r#"# TDD.md

Version: tdd-manual.v1

TDD is the test-first discipline used by the Build Agent execution pipeline.

## Rules

- Quality standards come from SPEC acceptance criteria.
- TDD does not redefine requirement quality.
- Build Agent is authorized only inside a complete Build Agent execution pipeline handoff.
- The test design stage must derive tests from the public requirement record and spec issue context before code changes.
- If TDD fits the task, Build Agent records the failing test before implementation.
- If TDD does not fit the task, Build Agent records why and defines the replacement smoke, build, screenshot, or command verification.
- Sandbox verification evidence must be recorded before PR/MR creation and Done writeback.

## Code Comment Language and Style

When Build Agent authors code inside the execution pipeline, any newly authored code comment, test comment, or doc comment MUST follow `agentLocale` and `plain-work-style`.

Do not rewrite existing comments only to change their language.

## V1 Boundary

V1 creates this manual and records the test discipline for the Build Agent execution pipeline.
"#;

const RELEASE_MANUAL: &str = r#"# RELEASE.md

Version: release-manual.v1

Release delivery is owned by Build Agent in V1.

There is no standalone Release Agent in V1.

## Purpose

RELEASE.md tells Build Agent how to prepare public delivery records after task run and evidence are available.

## Build Agent Release Delivery

After a successful task run, Build Agent may prepare:

- PR draft
- PR metadata
- Review checklist
- Changelog entry
- Release note
- Delivery record

These artifacts are written to the PR/MR body, CHANGELOG entry, or release notes. `.agentflow/tasks/<issue-id>/evidence/**` keeps local verification evidence only.

Task completion and audit are separate flows.

After Release Delivery exists, AgentFlow must not automatically create an audit request only because a task reached Done.

Audit starts only when an independent audit issue exists under:

`.agentflow/spec/issues/audit-<release-id>.json`

or when a human explicitly requests audit through an Agent conversation.

The audit issue is the primary entry for Audit Agent work. It must use `issueCategory=audit` and `requiredAgentRole=audit-agent`.

AgentFlow may keep `audit-request.json` as compatibility metadata only when an audit request already exists.

## V1 Boundary

Build Agent may prepare public delivery records.

Build Agent must not:

- merge
- deploy
- release to production
- run dangerous commands
- bypass high-risk confirmation
- modify public requirement records
- modify spec issue facts
- write audit reports
- execute audit issues
- create audit requests from task Done writeback

## Required Inputs

- spec issue
- public requirement record
- task run result
- task evidence
- changed-files summary
- validation result

## Audit Handoff

Build Agent stops after PR/MR merge and Done writeback are recorded.

Build Agent must not create audit requests when a task is done.

Audit Agent starts only from an independent audit issue or explicit human audit request.

The ordinary App UI only displays audit state and report material. It must not create audits.

## Public Delivery Content

- PR/MR body
- PR/MR metadata
- review checklist
- CHANGELOG entry
- release note
"#;

const AUDIT_MANUAL: &str = r#"# AUDIT.md

Version: audit-manual.v1

Audit is the code review and risk review working manual for future Audit Agent execution.

## Rules

- Audit Agent is enabled for Release Audit V1.
- Audit Agent completes existing `audit` issues and `human-via-agent` audit requests.
- Audit checks SPEC alignment, boundary compliance, architecture impact, permission / path / data-write risk, test coverage, retired-path reintroduction, unauthorized execution, unauthorized writes, model calls, and evidence completeness.
- Audit output belongs under `.agentflow/audit/`.
- The same Release Delivery must not have duplicate audit requests.
- Human conversation can ask an Agent for `human-via-agent` audit. The ordinary App UI must not create audits.

## Required Outputs

- audit.json
- audit-report.md
- findings.json
- checklist.md
- evidence-map.json
- traceability.json

## V1 Boundary

Audit Agent writes only audit artifacts for the selected audit request. It must not modify source code, spec facts, task artifacts, public delivery records, remote objects, or project commands.
"#;
