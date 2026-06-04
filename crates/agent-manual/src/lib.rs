pub mod model;

mod git;
mod hash;
mod layout;
mod lock;
mod manager;
mod repair;
mod templates;
mod validate;

pub use manager::{
    assert_agent_environment_ready, load_agent_environment_status, prepare_agent_working_manual,
};
pub use repair::repair_agent_working_manual;
pub use validate::validate_agent_working_manual;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AgentEnvironmentState;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn prepare_creates_agent_manual_tree() {
        let dir = tempdir().unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert_eq!(status.status, AgentEnvironmentState::Repaired);
        assert!(dir.path().join("AGENTS.md").is_file());
        assert!(!dir.path().join("AGENT.MD").exists());
        assert!(dir
            .path()
            .join(".agentflow/workspace-manifest.json")
            .is_file());
        assert!(status.workspace_manifest.valid);
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
        assert!(dir.path().join(".agentflow/spec/changes").is_dir());
        assert!(dir.path().join(".agentflow/goal-tree/goals").is_dir());
        assert!(dir.path().join(".agentflow/panel/context-packs").is_dir());
        assert!(dir.path().join(".agentflow/execute/commands").is_dir());
        assert!(dir.path().join(".agentflow/output/audit").is_dir());
        assert!(dir.path().join(".agentflow/state/health").is_dir());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills-lock.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills/request-triage/SKILL.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md")
            .is_file());
        assert_eq!(status.skills_lock.skill_count, 6);
        assert_eq!(status.skills.len(), 6);
        assert!(fs::read_to_string(dir.path().join("AGENTS.md"))
            .unwrap()
            .contains("requirement-intake-filter"));
        assert!(
            fs::read_to_string(dir.path().join(".agentflow/define/agent/Agentflow.md"))
                .unwrap()
                .contains("Requirement intake filter")
        );
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/bootstrap.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/validation.json")
            .is_file());
    }

    #[test]
    fn prepare_backs_up_existing_agents_md_before_rewrite() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# Existing\n").unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.agent_md.backed_up);
        assert!(fs::read_to_string(dir.path().join("AGENTS.md"))
            .unwrap()
            .contains("AGENTFLOW:MANAGED"));
        let backups = fs::read_dir(dir.path().join(".agentflow/output/backup/agent-md"))
            .unwrap()
            .count();
        assert_eq!(backups, 1);
    }

    #[test]
    fn existing_legacy_agent_md_is_compatibility_only() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("AGENT.MD"), "# Legacy\n").unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.ready);
        assert!(dir.path().join("AGENTS.md").is_file());
        assert_eq!(
            fs::read_to_string(dir.path().join("AGENT.MD")).unwrap(),
            "# Legacy\n"
        );
        assert!(status.legacy_agent_entry.exists);
        assert_eq!(status.legacy_agent_entry.path, "AGENT.MD");
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
        assert_eq!(repaired.skills_lock.skill_count, 6);
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
