pub mod model;

mod git;
mod hash;
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
        assert!(dir.path().join("AGENT.MD").is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/Agentflow.md")
            .is_file());
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
            .join(".agentflow/define/agent/state/bootstrap.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/state/validation.json")
            .is_file());
    }

    #[test]
    fn prepare_backs_up_existing_agent_md_before_rewrite() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("AGENT.MD"), "# Existing\n").unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert!(status.agent_md.backed_up);
        assert!(fs::read_to_string(dir.path().join("AGENT.MD"))
            .unwrap()
            .contains("AGENTFLOW:MANAGED"));
        let backups = fs::read_dir(dir.path().join(".agentflow/output/backup/agent-md"))
            .unwrap()
            .count();
        assert_eq!(backups, 1);
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
    fn tracked_agent_md_is_warning_not_blocker() {
        let dir = tempdir().unwrap();
        std::process::Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::write(dir.path().join("AGENT.MD"), "# Existing\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "AGENT.MD"])
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

    #[cfg(unix)]
    #[test]
    fn external_agent_md_symlink_blocks_repair() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let outside_agent = outside.path().join("AGENT.MD");
        fs::write(&outside_agent, "# outside\n").unwrap();
        symlink(&outside_agent, dir.path().join("AGENT.MD")).unwrap();

        let status = prepare_agent_working_manual(dir.path()).unwrap();

        assert_eq!(status.status, AgentEnvironmentState::Blocked);
        assert!(!status.ready);
        assert!(status
            .errors
            .iter()
            .any(|error| error.contains("symlink outside project root")));
    }
}
