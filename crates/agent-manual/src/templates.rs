use crate::model::{AGENT_ENTRY_VERSION, AGENT_MANUAL_VERSION, SKILL_VERSION};

pub const AGENT_MANUAL_RELATIVE_PATH: &str = ".agentflow/define/agent/Agentflow.md";
pub const SKILLS_LOCK_RELATIVE_PATH: &str = ".agentflow/define/agent/skills-lock.json";
pub const VALIDATION_RELATIVE_PATH: &str = ".agentflow/define/agent/state/validation.json";
pub const BOOTSTRAP_RELATIVE_PATH: &str = ".agentflow/define/agent/state/bootstrap.json";

#[derive(Debug, Clone, Copy)]
pub struct AgentSkillTemplate {
    pub name: &'static str,
    pub relative_path: &'static str,
    pub content: &'static str,
}

pub fn agent_md_template() -> String {
    format!(
        r#"# AGENT.MD

<!-- AGENTFLOW:MANAGED version={AGENT_ENTRY_VERSION} -->

This project is managed by AgentFlow.

Every Agent MUST read and follow:

1. `.agentflow/define/agent/Agentflow.md`
2. `.agentflow/define/agent/skills-lock.json`
3. All skills referenced by `skills-lock.json`

## Hard Rules

- Do not write source code unless AgentFlow rules explicitly allow it.
- Do not execute project commands unless AgentFlow rules explicitly allow it.
- Do not create or edit Goal Tree directly.
- Do not bypass OpenSpec.
- Do not create PRs, issues, or remote objects unless explicitly authorized.
- Human conversation is for confirmation and feedback, not direct Goal Tree editing.

## Current Flow

Conversation with human
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun

If any rule conflicts, AgentFlow rules win.

<!-- AGENTFLOW:END -->
"#
    )
}

pub fn agentflow_manual_template() -> String {
    format!(
        r#"# Agentflow.md

Version: {AGENT_MANUAL_VERSION}

## Role

You are an Agent working inside an AgentFlow-managed local project.

## Required Reading Order

1. `<project-root>/AGENT.MD`
2. `.agentflow/define/agent/Agentflow.md`
3. `.agentflow/define/agent/skills-lock.json`
4. All required skills listed in `skills-lock.json`

## Current Project Facts

- Project Workspace is local-first.
- `.agentflow/` is the local AgentFlow runtime and definition space.
- Goal Tree is agent-only and human read-only.
- OpenSpec is the requirement source.
- Goal Tree is derived from approved OpenSpec.
- AgentRun is not authorized yet.

## Allowed Actions

- Read project files.
- Read Graph status.
- Read Project File Reader metadata.
- Read Goal Tree snapshot.
- Read existing OpenSpec drafts / approvals when they exist.
- Ask human clarification questions.
- Produce OpenSpec Draft previews in conversation.

## Forbidden Actions

- Do not write user source code.
- Do not execute project commands.
- Do not run tests.
- Do not create or edit Goal Tree directly.
- Do not write approved OpenSpec without human confirmation.
- Do not start AgentRun.
- Do not create PRs or remote issues.
- Do not use legacy workflow paths.

## Required Workflow

Conversation
→ Request triage
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun

## OpenSpec First Rule

Feature, refactor, cleanup, and unclear change requests must go through OpenSpec Draft Preview before any Goal Tree materialization.

## Goal Tree Rule

Goal Tree is an agent-only derived fact source. Humans can inspect it through Desktop, but Desktop cannot write it.

## Execution Boundary

AgentRun is not authorized in this stage. Agents must stop before source writes, command execution, tests, PR creation, or remote issue creation.

## Validation Rule

Before any output or future write, the Agent must verify that AGENT.MD, Agentflow.md, skills-lock.json, boundary-check, and validation skills were read.

## Boundary

If the requested action is outside the current authorized stage, stop and ask for confirmation or wait for the next AgentFlow requirement.
"#
    )
}

pub fn skill_templates() -> [AgentSkillTemplate; 5] {
    [
        AgentSkillTemplate {
            name: "request-triage",
            relative_path: ".agentflow/define/agent/skills/request-triage/SKILL.md",
            content: REQUEST_TRIAGE_SKILL,
        },
        AgentSkillTemplate {
            name: "openspec-authoring",
            relative_path: ".agentflow/define/agent/skills/openspec-authoring/SKILL.md",
            content: OPENSPEC_AUTHORING_SKILL,
        },
        AgentSkillTemplate {
            name: "goal-tree-materialization",
            relative_path: ".agentflow/define/agent/skills/goal-tree-materialization/SKILL.md",
            content: GOAL_TREE_MATERIALIZATION_SKILL,
        },
        AgentSkillTemplate {
            name: "boundary-check",
            relative_path: ".agentflow/define/agent/skills/boundary-check/SKILL.md",
            content: BOUNDARY_CHECK_SKILL,
        },
        AgentSkillTemplate {
            name: "validation",
            relative_path: ".agentflow/define/agent/skills/validation/SKILL.md",
            content: VALIDATION_SKILL,
        },
    ]
}

pub fn skill_version() -> &'static str {
    SKILL_VERSION
}

const REQUEST_TRIAGE_SKILL: &str = r#"# request-triage

Version: v1

## Purpose

Classify the human request before any AgentFlow fact source is written.

## Categories

- bug
- feature
- refactor
- docs
- research
- cleanup
- question

## Rules

- feature: must enter OpenSpec Authoring.
- unclear change: ask questions first; do not write fact sources.
- bug: require reproduction information, current behavior, and expected behavior.
- cleanup: constrain scope and non-goals before planning.
- question: answer only; do not write fact sources.
- research: output findings only unless the human confirms entry into OpenSpec.
"#;

const OPENSPEC_AUTHORING_SKILL: &str = r#"# openspec-authoring

Version: v1

## Purpose

Generate an OpenSpec Draft Preview from conversation.

## Draft Preview Contents

- Summary
- Problem
- Goals
- Non-goals
- User behavior
- Edge cases
- Acceptance criteria
- Risks
- Open questions
- Product spec draft
- Tech spec draft
- Tasks draft
- Validation plan

## Hard Rules

- Without human confirmation, do not write `.agentflow/define/openspec/**`.
- OpenSpec is the requirement source.
- Goal Tree is a derived artifact.
"#;

const GOAL_TREE_MATERIALIZATION_SKILL: &str = r#"# goal-tree-materialization

Version: v1

## Purpose

Convert approved OpenSpec into Goal / Milestone / Issue definitions.

## Hard Rules

- Do not generate Goal Tree from chat directly.
- Generate only from approved OpenSpec.
- Writes must use `agent-system`.
- Goal Tree is human read-only.
- Goal Tree does not execute.

## Mapping

- OpenSpec objective -> Goal.objective
- OpenSpec scope / non-goals -> Goal.scope / Goal.nonGoals
- OpenSpec phases / design stages -> Milestone.stageGoal
- OpenSpec tasks -> Issue.goal
- OpenSpec acceptance criteria -> Issue.acceptanceCriteria
- OpenSpec constraints / boundaries -> Issue.boundary
- OpenSpec task dependencies -> Issue.dependencies
"#;

const BOUNDARY_CHECK_SKILL: &str = r#"# boundary-check

Version: v1

## Purpose

Check every Agent action before it proceeds.

## Checks

- Is the Agent about to write user source code?
- Is the Agent about to execute a command?
- Is the Agent about to write OpenSpec?
- Has human confirmation been captured?
- Is the Agent about to write Goal Tree?
- Does approved OpenSpec exist?
- Is the Agent about to start AgentRun?
- Is the Agent about to create a remote object?
- Does the action touch legacy paths?

## If Out Of Bounds

Stop, explain the reason, and request human confirmation or wait for the next AgentFlow capability.
"#;

const VALIDATION_SKILL: &str = r#"# validation

Version: v1

## Purpose

Self-check before any Agent output or future write.

## Checks

- Was AGENT.MD read?
- Was Agentflow.md read?
- Was skills-lock.json read?
- Is OpenSpec-first preserved?
- Did the Agent avoid erroneous Goal Tree writes?
- Did the Agent avoid unauthorized command execution?
- Are there unresolved confirmation questions?
- Should the Agent stop?
"#;
