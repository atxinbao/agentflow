pub mod model;

mod git;
mod hash;
mod layout;
mod locale;
mod lock;
mod manager;
mod ownership;
mod repair;
mod style;
mod templates;
mod validate;

pub use manager::{
    assert_agent_environment_ready, load_agent_environment_status, prepare_agent_working_manual,
    prepare_agent_working_manual_with_locale,
};
pub use ownership::{
    assert_agentflow_workspace_owned_or_creatable, check_agentflow_workspace_ownership,
    take_over_agentflow_workspace,
};
pub use repair::{repair_agent_working_manual, repair_agent_working_manual_with_locale};
pub use validate::validate_agent_working_manual;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AgentEnvironmentState, WorkspaceOwnershipState};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn prepare_creates_agent_manual_tree() {
        let dir = tempdir().unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert_eq!(status.status, AgentEnvironmentState::Repaired);
        assert!(dir.path().join("AGENTS.md").is_file());
        assert!(dir
            .path()
            .join(".agentflow/workspace-manifest.json")
            .is_file());
        assert!(status.workspace_manifest.valid);
        assert_eq!(
            status.ownership.status,
            WorkspaceOwnershipState::ManagedCurrent
        );
        assert!(status.layout.ready);
        assert!(dir
            .path()
            .join(".agentflow/define/agent/Agentflow.md")
            .is_file());
        assert!(dir.path().join(".agentflow/define/spec/SPEC.md").is_file());
        assert!(dir.path().join(".agentflow/define/tdd/TDD.md").is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/release/RELEASE.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/audit/AUDIT.md")
            .is_file());
        assert!(!dir.path().join(".agentflow/input").exists());
        assert!(dir.path().join(".agentflow/spec/projects").is_dir());
        assert!(dir.path().join(".agentflow/spec/issues").is_dir());
        assert!(dir.path().join(".agentflow/events").is_dir());
        assert!(dir.path().join(".agentflow/projections/tasks").is_dir());
        assert!(dir.path().join(".agentflow/projections/projects").is_dir());
        assert!(dir.path().join(".agentflow/tasks").is_dir());
        assert!(!dir.path().join(".agentflow/goal-tree").exists());
        assert!(dir.path().join(".agentflow/panel/context-packs").is_dir());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(dir.path().join(".agentflow/audit").is_dir());
        assert!(!dir.path().join(".agentflow/output/release").exists());
        assert!(dir.path().join(".agentflow/state/health").is_dir());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills-lock.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/roles.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills/request-triage/SKILL.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills/spec-issue-generation/SKILL.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills/plain-work-style/SKILL.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/locale.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/style.json")
            .is_file());
        assert_eq!(status.skills_lock.skill_count, 7);
        assert_eq!(status.skills.len(), 7);
        assert!(!status.locale.agent_locale.is_empty());
        assert_eq!(status.locale.manual_language, "en");
        assert_eq!(status.style.style_id, "plain-work-style");
        assert!(status.style.applies_to_code_comments);
        assert!(fs::read_to_string(dir.path().join("AGENTS.md"))
            .unwrap()
            .contains("requirement-intake-filter"));
        assert!(fs::read_to_string(dir.path().join("AGENTS.md"))
            .unwrap()
            .contains("separate Codex threads for Spec Agent, Build Agent, and Audit Agent"));
        assert!(
            fs::read_to_string(dir.path().join(".agentflow/define/agent/Agentflow.md"))
                .unwrap()
                .contains("Requirement intake filter")
        );
        assert!(
            fs::read_to_string(dir.path().join(".agentflow/define/agent/Agentflow.md"))
                .unwrap()
                .contains("AgentFlow / Build Agent")
        );
        assert!(
            fs::read_to_string(dir.path().join(".agentflow/define/agent/Agentflow.md"))
                .unwrap()
                .contains("Build Agent execution pipeline")
        );
        let agentflow_manual =
            fs::read_to_string(dir.path().join(".agentflow/define/agent/Agentflow.md")).unwrap();
        assert!(agentflow_manual.contains("Test design"));
        assert!(agentflow_manual.contains("keep the issue in `in_review`"));
        assert!(agentflow_manual.contains(
            "issue dependencies belong in each spec issue contract through `blockedBy`."
        ));
        assert!(agentflow_manual.contains(
            "Do not create extra relation files, and do not use old `from` / `to` relation fields."
        ));
        assert!(
            fs::read_to_string(dir.path().join(".agentflow/define/audit/AUDIT.md"))
                .unwrap()
                .contains("Audit Agent is enabled for Release Audit V1")
        );
        let roles =
            fs::read_to_string(dir.path().join(".agentflow/define/agent/roles.json")).unwrap();
        assert!(roles.contains("\"agentRole\": \"spec-agent\""));
        assert!(roles.contains("\"agentRole\": \"build-agent\""));
        assert!(roles.contains("\"agentRole\": \"audit-agent\""));
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/bootstrap.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/validation.json")
            .is_file());
        let spec_manual =
            fs::read_to_string(dir.path().join(".agentflow/define/spec/SPEC.md")).unwrap();
        assert!(spec_manual.contains("`docs/requirements/**`"));
        assert!(spec_manual.contains("`.agentflow/spec/projects/**`"));
        assert!(spec_manual.contains("`.agentflow/spec/issues/**`"));
        assert!(spec_manual.contains("Do not show raw JSON as the default conversation output."));
        assert!(spec_manual.contains("`docs/requirements/<requirement-id>.md`"));
        assert!(spec_manual.contains(
            "Spec Project and Issue human-facing fields must follow the user's `agentLocale`."
        ));
        assert!(!spec_manual.contains("`.agentflow/input/specs/`"));
        let intake_skill = fs::read_to_string(
            dir.path()
                .join(".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md"),
        )
        .unwrap();
        assert!(intake_skill.contains("Do not show raw JSON as the default conversation output."));
        assert!(intake_skill.contains("## Internal Record Shape"));
        let spec_authoring_skill = fs::read_to_string(
            dir.path()
                .join(".agentflow/define/agent/skills/spec-gate-authoring/SKILL.md"),
        )
        .unwrap();
        assert!(spec_authoring_skill.contains("SPEC Draft Preview must be human-readable"));
        assert!(spec_authoring_skill.contains("`docs/requirements/<requirement-id>.md`"));
        assert!(spec_authoring_skill.contains(
            "Spec project / issue contracts write only to `.agentflow/spec/projects/**`"
        ));
        let spec_issue_generation_skill = fs::read_to_string(
            dir.path()
                .join(".agentflow/define/agent/skills/spec-issue-generation/SKILL.md"),
        )
        .unwrap();
        assert!(spec_issue_generation_skill.contains("# spec-issue-generation"));
        assert!(spec_issue_generation_skill
            .contains("Generate only from `docs/requirements/<requirement-id>.md`."));
        assert!(spec_issue_generation_skill.contains("`.agentflow/spec/issues/<issue-id>.json`"));
        assert!(spec_issue_generation_skill
            .contains("Dependencies belong in each spec issue contract through `blockedBy`."));
        assert!(spec_issue_generation_skill.contains(
            "Project and Issue human-facing natural-language fields MUST follow the current `agentLocale`."
        ));
    }

    #[test]
    fn prepare_records_agent_locale_and_manual_language() {
        let dir = tempdir().unwrap();

        let status =
            prepare_agent_working_manual_with_locale(dir.path(), Some("zh_CN".to_string()))
                .unwrap();

        assert!(status.ready);
        assert_eq!(status.locale.agent_locale, "zh-CN");
        assert_eq!(status.locale.raw_os_locale.as_deref(), Some("zh_CN"));
        assert_eq!(status.locale.manual_language, "en");
        assert_eq!(status.locale.source, "app");
        assert_eq!(status.style.style_id, "plain-work-style");

        let locale_json =
            fs::read_to_string(dir.path().join(".agentflow/define/agent/state/locale.json"))
                .unwrap();
        assert!(locale_json.contains("\"agentLocale\": \"zh-CN\""));
        assert!(locale_json.contains("\"manualLanguage\": \"en\""));
    }

    #[test]
    fn skills_lock_records_agent_locale_and_style_policy() {
        let dir = tempdir().unwrap();

        prepare_agent_working_manual_with_locale(dir.path(), Some("ja_JP".to_string())).unwrap();

        let lock = fs::read_to_string(dir.path().join(".agentflow/define/agent/skills-lock.json"))
            .unwrap();
        assert!(lock.contains("\"manualLanguage\": \"en\""));
        assert!(lock.contains("\"agentLocale\": \"ja-JP\""));
        assert!(lock.contains("\"styleId\": \"plain-work-style\""));
        assert!(lock.contains("\"appliesToCodeComments\": true"));
    }

    #[test]
    fn locale_metadata_changes_do_not_rewrite_manual_templates() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual_with_locale(dir.path(), Some("en_US".to_string())).unwrap();
        let agent_entry_before = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
        let skill_before = fs::read_to_string(
            dir.path()
                .join(".agentflow/define/agent/skills/plain-work-style/SKILL.md"),
        )
        .unwrap();

        let status =
            prepare_agent_working_manual_with_locale(dir.path(), Some("zh_CN".to_string()))
                .unwrap();

        assert!(status.ready);
        assert_eq!(status.locale.agent_locale, "zh-CN");
        assert_eq!(
            fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
            agent_entry_before
        );
        assert_eq!(
            fs::read_to_string(
                dir.path()
                    .join(".agentflow/define/agent/skills/plain-work-style/SKILL.md")
            )
            .unwrap(),
            skill_before
        );
        assert!(!status
            .errors
            .iter()
            .any(|error| error.contains("plain-work-style hash mismatch")));
    }

    #[test]
    fn spec_agent_status_allows_spec_facts_after_confirmation() {
        let manual = crate::templates::agentflow_manual_template();

        assert!(manual.contains("Status: enabled for Spec Contract V1."));
        assert!(manual.contains(
            "After confirmation, it may write public requirement records under `docs/requirements/**`"
        ));
        assert!(manual.contains("spec project / issue contracts under `.agentflow/spec/**`"));
        assert!(manual.contains("Generated spec issues must use `issueCategory=spec`"));
        assert!(manual.contains("Do not write AgentFlow facts outside the current role boundary."));
        assert!(!manual.contains("Status: enabled.\n\nCombines requirement intake"));
    }

    #[test]
    fn agent_roles_consolidate_release_into_build_agent() {
        let manual = crate::templates::agentflow_manual_template();

        assert!(manual.contains("### 1. Spec Agent"));
        assert!(manual.contains("### 2. Build Agent"));
        assert!(manual.contains("### 3. Audit Agent"));
        assert!(manual.contains("three separate Codex threads"));
        assert!(
            manual.contains("A Codex thread must not switch from development work to audit work")
        );
        assert!(!manual.contains("### 3. Release Agent"));
        assert!(!manual.contains("### 4. Audit Agent"));
        assert!(manual.contains("Status: enabled for Execute + Release Delivery V1."));
        assert!(manual.contains(".agentflow/tasks/<issue-id>/runs/<run-id>/"));
        assert!(manual.contains(".agentflow/tasks/<issue-id>/evidence/**"));
        assert!(
            manual.contains("public delivery records in PR/MR body, CHANGELOG, or release notes")
        );
        assert!(manual.contains(
            "The writeback stage runs only after PR/MR merge and writes run events, task evidence, public delivery references, and `done` status."
        ));
        assert!(manual.contains("The current AgentFlow spec issue is the only task authority."));
        assert!(manual.contains("The handoff package is only a derived snapshot for transport"));
        assert!(
            manual.contains("`executionPipeline` is only one field inside the spec issue contract")
        );
        assert!(manual.contains(
            "Build Agent must use the current AgentFlow spec issue as the only task authority."
        ));
        assert!(manual.contains(
            "It must not treat any external issue, task, plan, queue, thread, or tool state as task authority."
        ));
        assert!(manual.contains(
            "`Do not handwrite .agentflow/**` means Build Agent must not directly edit AgentFlow facts by hand;"
        ));
        assert!(manual.contains("Issue preflight"));
        assert!(manual.contains("After preflight passes, AgentFlow moves the issue to `todo`"));
        assert!(manual.contains(
            "Runtime preflight must then create the current run through the official AgentFlow run loop before any source code change starts."
        ));
        assert!(manual.contains(
            "After runtime preflight confirms the Panel Context Pack is readable or successfully generated and the current run has been created"
        ));
        assert!(manual.contains("AgentFlow moves the current run to `planned`"));
        assert!(manual.contains("auto-merge-if-eligible"));
        assert!(manual.contains("A Draft PR/MR is only an intermediate state"));
        assert!(manual.contains("gh pr merge --auto"));
        assert!(manual.contains("glab mr merge --auto-merge"));
        assert!(!manual.contains("`release-auto` audit request"));
    }

    #[test]
    fn release_manual_is_build_agent_delivery_manual() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        let release_manual =
            fs::read_to_string(dir.path().join(".agentflow/define/release/RELEASE.md")).unwrap();

        assert!(release_manual.contains("Release delivery is owned by Build Agent in V1."));
        assert!(release_manual.contains("There is no standalone Release Agent in V1."));
        assert!(release_manual.contains(".agentflow/tasks/<issue-id>/evidence/**"));
        assert!(release_manual.contains("PR/MR body, CHANGELOG entry, or release notes"));
        assert!(release_manual.contains("must not automatically create an audit request"));
        assert!(!release_manual.contains("future Release Agent execution"));
        assert!(!release_manual.contains("Release Agent is currently not authorized yet"));
        assert!(!release_manual.contains("release-auto"));
    }

    #[test]
    fn prepare_keeps_existing_local_agents_md_without_rewrite() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# Existing\n").unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert!(!status.agent_md.backed_up);
        assert!(!status.agent_md.managed);
        assert_eq!(
            fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
            "# Existing\n"
        );
        assert_eq!(
            fs::read_dir(dir.path().join(".agentflow/define/agent/backup/agent-md"))
                .unwrap()
                .count(),
            0
        );
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("not AgentFlow-managed")));
        assert!(status
            .repairs
            .iter()
            .any(|repair| repair.contains("Kept existing local AGENTS.md")));
    }

    #[test]
    fn validate_detects_skill_hash_mismatch_and_repair_restores_it() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        let skill_path = dir
            .path()
            .join(".agentflow/define/agent/skills/validation/SKILL.md");
        fs::write(&skill_path, "# tampered\n").unwrap();

        let invalid = validate_agent_working_manual(dir.path()).unwrap();
        assert!(!invalid.ready);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("validation hash mismatch")));

        let repaired = repair_agent_working_manual(dir.path()).unwrap();
        assert!(repaired.ready);
        assert!(fs::read_to_string(skill_path)
            .unwrap()
            .contains("Self-check"));
    }

    #[test]
    fn validate_detects_missing_requirement_intake_filter_skill() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        let skill_path = dir
            .path()
            .join(".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md");
        fs::remove_file(&skill_path).unwrap();

        let invalid = validate_agent_working_manual(dir.path()).unwrap();

        assert!(!invalid.ready);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("Skill requirement-intake-filter is missing")));

        let repaired = repair_agent_working_manual(dir.path()).unwrap();

        assert!(repaired.ready);
        assert!(skill_path.is_file());
        assert_eq!(repaired.skills_lock.skill_count, 7);
    }

    #[test]
    fn validate_detects_missing_locale_state_and_repair_restores_it() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual_with_locale(dir.path(), Some("zh_CN".to_string())).unwrap();
        let locale_path = dir.path().join(".agentflow/define/agent/state/locale.json");
        fs::remove_file(&locale_path).unwrap();

        let invalid = crate::validate::validate_agent_working_manual_with_context(
            dir.path(),
            Vec::new(),
            None,
            Some("zh_CN"),
        )
        .unwrap();

        assert!(!invalid.ready);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("locale state is missing")));

        let repaired =
            repair_agent_working_manual_with_locale(dir.path(), Some("zh_CN".to_string())).unwrap();

        assert!(repaired.ready);
        assert!(locale_path.is_file());
        assert_eq!(repaired.locale.agent_locale, "zh-CN");
    }

    #[test]
    fn validate_detects_missing_style_state_and_repair_restores_it() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        let style_path = dir.path().join(".agentflow/define/agent/state/style.json");
        fs::remove_file(&style_path).unwrap();

        let invalid = validate_agent_working_manual(dir.path()).unwrap();

        assert!(!invalid.ready);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("style state is missing")));

        let repaired = repair_agent_working_manual(dir.path()).unwrap();

        assert!(repaired.ready);
        assert!(style_path.is_file());
        assert_eq!(repaired.style.style_id, "plain-work-style");
    }

    #[test]
    fn validate_detects_missing_plain_work_style_skill_and_repair_restores_it() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        let skill_path = dir
            .path()
            .join(".agentflow/define/agent/skills/plain-work-style/SKILL.md");
        fs::remove_file(&skill_path).unwrap();

        let invalid = validate_agent_working_manual(dir.path()).unwrap();

        assert!(!invalid.ready);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("Skill plain-work-style is missing")));

        let repaired = repair_agent_working_manual(dir.path()).unwrap();

        assert!(repaired.ready);
        assert!(skill_path.is_file());
        assert!(fs::read_to_string(skill_path)
            .unwrap()
            .contains("Default Output Structure"));
    }

    #[test]
    fn code_comment_policy_is_present_in_agentflow_and_tdd_manuals() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();

        let agentflow =
            fs::read_to_string(dir.path().join(".agentflow/define/agent/Agentflow.md")).unwrap();
        let tdd = fs::read_to_string(dir.path().join(".agentflow/define/tdd/TDD.md")).unwrap();

        assert!(agentflow.contains("newly authored code comments"));
        assert!(agentflow.contains("Do not mass-translate existing code comments."));
        assert!(tdd.contains("Code Comment Language and Style"));
        assert!(tdd.contains("test design stage"));
        assert!(tdd.contains("agentLocale"));
        assert!(tdd.contains("plain-work-style"));
    }

    #[test]
    fn validate_detects_missing_state_files() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        fs::remove_file(
            dir.path()
                .join(".agentflow/define/agent/state/bootstrap.json"),
        )
        .unwrap();
        fs::remove_file(
            dir.path()
                .join(".agentflow/define/agent/state/validation.json"),
        )
        .unwrap();

        let invalid = validate_agent_working_manual(dir.path()).unwrap();

        assert!(!invalid.ready);
        assert_eq!(invalid.status, AgentEnvironmentState::Missing);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("bootstrap state is missing")));
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("validation state is missing")));
    }

    #[test]
    fn ownership_none_is_creatable() {
        let dir = tempdir().unwrap();

        let ownership = check_agentflow_workspace_ownership(dir.path()).unwrap();

        assert_eq!(ownership.status, WorkspaceOwnershipState::None);
        assert!(ownership.ready_for_prepare);
        assert!(!ownership.agent_blocked);
    }

    #[test]
    fn ownership_foreign_blocks_prepare_without_writing() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(dir.path().join(".agentflow/custom.txt"), "foreign\n").unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert_eq!(status.status, AgentEnvironmentState::Blocked);
        assert_eq!(status.ownership.status, WorkspaceOwnershipState::Foreign);
        assert!(!dir.path().join("AGENTS.md").exists());
        assert!(!dir
            .path()
            .join(".agentflow/workspace-manifest.json")
            .exists());
        assert_eq!(
            fs::read_to_string(dir.path().join(".agentflow/custom.txt")).unwrap(),
            "foreign\n"
        );
    }

    #[test]
    fn ownership_corrupted_agentflow_manifest_can_be_repaired_when_marker_exists() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(
            dir.path().join(".agentflow/workspace-manifest.json"),
            "{ AgentFlow",
        )
        .unwrap();

        let before = check_agentflow_workspace_ownership(dir.path()).unwrap();
        assert_eq!(before.status, WorkspaceOwnershipState::Corrupted);

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert_eq!(
            status.ownership.status,
            WorkspaceOwnershipState::ManagedCurrent
        );
    }

    #[test]
    fn ownership_corrupted_foreign_manifest_blocks_prepare() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(
            dir.path().join(".agentflow/workspace-manifest.json"),
            "{ nope",
        )
        .unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert_eq!(status.status, AgentEnvironmentState::Blocked);
        assert_eq!(status.ownership.status, WorkspaceOwnershipState::Blocked);
        assert!(!dir.path().join("AGENTS.md").exists());
    }

    #[test]
    fn takeover_renames_foreign_agentflow_and_creates_managed_workspace() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(dir.path().join(".agentflow/custom.txt"), "foreign\n").unwrap();

        let ownership = take_over_agentflow_workspace(dir.path()).unwrap();

        assert_eq!(ownership.status, WorkspaceOwnershipState::ManagedCurrent);
        assert!(dir
            .path()
            .join(".agentflow/workspace-manifest.json")
            .is_file());
        assert!(fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with(".agentflow.unmanaged.")));
    }

    #[cfg(unix)]
    #[test]
    fn internal_agentflow_symlink_warns_without_blocking() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let target = dir.path().join("agentflow-real");
        fs::create_dir_all(&target).unwrap();
        symlink(&target, dir.path().join(".agentflow")).unwrap();

        let ownership = check_agentflow_workspace_ownership(dir.path()).unwrap();

        assert_ne!(ownership.status, WorkspaceOwnershipState::Blocked);
        assert!(ownership
            .warnings
            .iter()
            .any(|warning| warning.contains("symlink inside project root")));
    }

    #[cfg(unix)]
    #[test]
    fn external_agentflow_symlink_blocks_prepare() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        symlink(outside.path(), dir.path().join(".agentflow")).unwrap();

        let ownership = check_agentflow_workspace_ownership(dir.path()).unwrap();

        assert_eq!(ownership.status, WorkspaceOwnershipState::Blocked);
        assert!(ownership.agent_blocked);
        assert!(ownership
            .errors
            .iter()
            .any(|error| error.contains("symlink outside project root")));
    }

    #[test]
    fn load_status_revalidates_when_bootstrap_state_is_missing() {
        let dir = tempdir().unwrap();
        prepare_agent_working_manual(dir.path()).unwrap();
        fs::remove_file(
            dir.path()
                .join(".agentflow/define/agent/state/bootstrap.json"),
        )
        .unwrap();

        let invalid = load_agent_environment_status(dir.path()).unwrap();

        assert!(!invalid.ready);
        assert_eq!(invalid.status, AgentEnvironmentState::Missing);
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.contains("bootstrap state is missing")));
    }

    #[test]
    fn tracked_agents_md_is_warning_not_blocker() {
        let dir = tempdir().unwrap();
        std::process::Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# Existing\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "AGENTS.md"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert!(status.agent_md.tracked_by_git);
        assert_eq!(
            fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
            "# Existing\n"
        );
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("tracked by Git")));
    }

    #[test]
    fn shadow_files_are_warning_not_blocker() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "# Other tool rules\n").unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert!(status
            .shadow_guard
            .detected
            .iter()
            .any(|path| path == "CLAUDE.md"));
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("Agent entry shadow detected: CLAUDE.md")));
    }

    #[cfg(unix)]
    #[test]
    fn internal_agents_md_symlink_warns_without_blocking() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let target = dir.path().join("managed-agent-entry.md");
        fs::write(&target, "# internal symlink target\n").unwrap();
        symlink(&target, dir.path().join("AGENTS.md")).unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert_ne!(status.status, AgentEnvironmentState::Blocked);
        assert!(fs::symlink_metadata(dir.path().join("AGENTS.md"))
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("symlink inside project root")));
    }

    #[cfg(unix)]
    #[test]
    fn external_agents_md_symlink_blocks_repair() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let outside_agent = outside.path().join("AGENTS.md");
        fs::write(&outside_agent, "# outside\n").unwrap();
        symlink(&outside_agent, dir.path().join("AGENTS.md")).unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert_eq!(status.status, AgentEnvironmentState::Blocked);
        assert!(!status.ready);
        assert!(status
            .errors
            .iter()
            .any(|error| error.contains("symlink outside project root")));
    }
}
