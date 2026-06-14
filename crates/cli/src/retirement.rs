//! Legacy CLI retirement policy.
//!
//! The archived 2026-05 workflow command names still parse so users get an
//! explicit retirement message instead of accidentally running stale writers.

use crate::args::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegacyCommandDisposition {
    KeepTemporary,
    DisableWithMessage,
    Delete,
}

impl LegacyCommandDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::KeepTemporary => "keep-temporary",
            Self::DisableWithMessage => "disable-with-message",
            Self::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegacyCommandStatus {
    pub(crate) command_name: &'static str,
    pub(crate) disposition: LegacyCommandDisposition,
    pub(crate) reason: &'static str,
}

impl LegacyCommandStatus {
    pub(crate) const fn disabled(command_name: &'static str, reason: &'static str) -> Self {
        Self {
            command_name,
            disposition: LegacyCommandDisposition::DisableWithMessage,
            reason,
        }
    }
}

pub(crate) fn legacy_command_status(command: &Command) -> LegacyCommandStatus {
    match command {
        Command::Metrics => LegacyCommandStatus {
            command_name: "metrics",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "temporary read-only compatibility command",
        },
        Command::Projects => LegacyCommandStatus {
            command_name: "projects",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "temporary read-only compatibility command",
        },
        Command::Search { .. } => LegacyCommandStatus {
            command_name: "search",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "temporary read-only compatibility command",
        },
        Command::Init { .. } => LegacyCommandStatus {
            command_name: "init",
            disposition: LegacyCommandDisposition::Delete,
            reason: "old goal bootstrap was removed with the legacy goal-tree crate",
        },
        Command::Context => LegacyCommandStatus {
            command_name: "context",
            disposition: LegacyCommandDisposition::Delete,
            reason: "old context writer is superseded by Panel read models",
        },
        Command::Goal { .. } => LegacyCommandStatus::disabled(
            "goal",
            "old GoalLoop is not inherited by the new requirements track",
        ),
        Command::Feature { .. } => LegacyCommandStatus::disabled(
            "feature",
            "old Product Feature flow is not inherited by the new requirements track",
        ),
        Command::Team { .. } => LegacyCommandStatus::disabled(
            "team",
            "old Team writer is not authorized by the new product model",
        ),
        Command::Milestone { .. } => LegacyCommandStatus::disabled(
            "milestone",
            "old Milestone writer is not authorized by the new product model",
        ),
        Command::Issue { .. } => LegacyCommandStatus::disabled(
            "issue",
            "old IssueContract writer is not authorized by the new product model",
        ),
        Command::Plan { .. } => {
            LegacyCommandStatus::disabled("plan", "old issue planning writer is not authorized")
        }
        Command::Run { .. } => {
            LegacyCommandStatus::disabled("run", "new AgentRun has not been defined yet")
        }
        Command::Verify { .. } => {
            LegacyCommandStatus::disabled("verify", "old verification writer is not authorized")
        }
        Command::Review { .. } => {
            LegacyCommandStatus::disabled("review", "old review/evidence writer is not authorized")
        }
        Command::Index { .. } => {
            LegacyCommandStatus::disabled("index", "old SQLite index writer is not authorized")
        }
        Command::View { .. } => {
            LegacyCommandStatus::disabled("view", "old saved-view writer is not authorized")
        }
        Command::Update { .. } => {
            LegacyCommandStatus::disabled("update", "old project summary writer is not authorized")
        }
        Command::Eligibility { .. } => LegacyCommandStatus::disabled(
            "eligibility",
            "old eligibility snapshot writer is not inherited",
        ),
        Command::Lease => {
            LegacyCommandStatus::disabled("lease", "old lease snapshot writer is not inherited")
        }
        Command::Project { .. } => LegacyCommandStatus::disabled(
            "project",
            "old project writer and closure/audit commands are not inherited",
        ),
        Command::ProjectSeed { .. } => LegacyCommandStatus::disabled(
            "project-seed",
            "old local project seed writer is not authorized",
        ),
        Command::IssueLink { .. } => {
            LegacyCommandStatus::disabled("issue-link", "old issue link writer is not authorized")
        }
        Command::ReviewAssistant { .. } => LegacyCommandStatus::disabled(
            "review-assistant",
            "old review assistant writer is not authorized",
        ),
        Command::BuildAgent { .. } => LegacyCommandStatus {
            command_name: "build-agent",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "active Build Agent completion writeback command",
        },
        Command::TaskLoop { .. } => LegacyCommandStatus {
            command_name: "task-loop",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "active task workflow scheduler and launcher command",
        },
        Command::AgentBridge { .. } => LegacyCommandStatus {
            command_name: "agent-bridge",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "active external agent launch consumer command",
        },
        Command::Projection { .. } => LegacyCommandStatus {
            command_name: "projection",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "active task/project projection read model command",
        },
        Command::Release { .. } => LegacyCommandStatus {
            command_name: "release",
            disposition: LegacyCommandDisposition::KeepTemporary,
            reason: "active public release document generator command",
        },
        Command::State { .. } => LegacyCommandStatus::disabled(
            "state",
            "old workflow state snapshot writer is not inherited",
        ),
    }
}

pub(crate) fn should_disable_legacy_command(command: &Command) -> bool {
    !matches!(
        legacy_command_status(command).disposition,
        LegacyCommandDisposition::KeepTemporary
    )
}

pub(crate) fn print_legacy_retirement_message(status: &LegacyCommandStatus) {
    println!("legacy command: {}", status.command_name);
    println!("disposition: {}", status.disposition.as_str());
    println!("reason: {}", status.reason);
    println!("This command belongs to the archived 2026-05 AgentFlow workflow.");
    println!("It is disabled in the new requirements track.");
    println!(
        "Use the current Project Workspace, Input, Execute, Output, and State workflow instead."
    );
    println!("No files were written and no command was executed.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::Command;

    #[test]
    fn read_only_commands_remain_temporarily_available() {
        assert_eq!(
            legacy_command_status(&Command::Metrics).disposition,
            LegacyCommandDisposition::KeepTemporary
        );
        assert_eq!(
            legacy_command_status(&Command::Projects).disposition,
            LegacyCommandDisposition::KeepTemporary
        );
        assert_eq!(
            legacy_command_status(&Command::Search {
                query: vec!["AgentFlow".to_string()]
            })
            .disposition,
            LegacyCommandDisposition::KeepTemporary
        );
        assert_eq!(
            legacy_command_status(&Command::BuildAgent {
                command: crate::args::BuildAgentCommand::Complete {
                    request: std::path::PathBuf::from("request.json")
                }
            })
            .disposition,
            LegacyCommandDisposition::KeepTemporary
        );
    }

    #[test]
    fn task_runtime_commands_remain_available() {
        assert_eq!(
            legacy_command_status(&Command::TaskLoop {
                command: crate::args::TaskLoopCommand::Tick {
                    project_id: "project".to_string(),
                    provider: "codex".to_string(),
                }
            })
            .disposition,
            LegacyCommandDisposition::KeepTemporary
        );
        assert_eq!(
            legacy_command_status(&Command::AgentBridge {
                command: crate::args::AgentBridgeCommand::ClaimNext
            })
            .disposition,
            LegacyCommandDisposition::KeepTemporary
        );
        assert_eq!(
            legacy_command_status(&Command::Projection {
                command: crate::args::ProjectionCommand::Rebuild
            })
            .disposition,
            LegacyCommandDisposition::KeepTemporary
        );
        assert_eq!(
            legacy_command_status(&Command::Release {
                command: crate::args::ReleaseCommand::Summary
            })
            .disposition,
            LegacyCommandDisposition::KeepTemporary
        );
    }

    #[test]
    fn old_writer_commands_are_disabled() {
        assert!(should_disable_legacy_command(&Command::Run {
            issue_id: "ISSUE-1".to_string(),
            dry_run: true,
        }));
        assert!(should_disable_legacy_command(&Command::Context));
    }
}
