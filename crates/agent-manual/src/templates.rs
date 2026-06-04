use crate::model::{AGENT_ENTRY_VERSION, AGENT_MANUAL_VERSION, SKILL_VERSION};

pub const AGENT_ENTRY_RELATIVE_PATH: &str = "AGENTS.md";
pub const LEGACY_AGENT_ENTRY_RELATIVE_PATH: &str = "AGENT.MD";
pub const AGENT_MANUAL_RELATIVE_PATH: &str = ".agentflow/define/agent/Agentflow.md";
pub const SKILLS_LOCK_RELATIVE_PATH: &str = ".agentflow/define/agent/skills-lock.json";
pub const VALIDATION_RELATIVE_PATH: &str = ".agentflow/define/agent/state/validation.json";
pub const BOOTSTRAP_RELATIVE_PATH: &str = ".agentflow/define/agent/state/bootstrap.json";
pub const WORKSPACE_MANIFEST_RELATIVE_PATH: &str = ".agentflow/workspace-manifest.json";

#[derive(Debug, Clone, Copy)]
pub struct AgentSkillTemplate {
    pub name: &'static str,
    pub relative_path: &'static str,
    pub content: &'static str,
}

pub fn agent_entry_template() -> String {
    format!(
        r#"# AGENTS.md

<!-- AGENTFLOW:MANAGED version={AGENT_ENTRY_VERSION} -->

This project is managed by AgentFlow.

Every Agent MUST read and follow:

1. `.agentflow/define/agent/Agentflow.md`
2. `.agentflow/define/agent/skills-lock.json`
3. All skills referenced by `skills-lock.json`

## Hard Rules

- Do not write source code unless AgentFlow rules explicitly allow it.
- Do not execute project commands unless AgentFlow rules explicitly allow it.
- Before producing an OpenSpec Draft, every Agent MUST run the requirement-intake-filter skill.
- Do not create or edit Goal Tree directly.
- Do not bypass SPEC.
- Do not create PRs, issues, or remote objects unless explicitly authorized.
- Human conversation is for confirmation and feedback, not direct Goal Tree editing.

## Current Flow

Conversation with human
→ Request triage
→ Requirement intake filter
→ SPEC Draft Preview
→ Human confirmation
→ Approved SPEC
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

1. `<project-root>/AGENTS.md`
2. `.agentflow/define/agent/Agentflow.md`
3. `.agentflow/define/agent/skills-lock.json`
4. All required skills listed in `skills-lock.json`

## Current Project Facts

- Project Workspace is local-first.
- `.agentflow/` is the local Agent workflow control plane.
- `define/` contains Agent manuals, templates, and skill definitions only.
- `AGENTS.md` is the canonical root Agent entry.
- `AGENT.MD` is legacy compatibility only.
- Goal Tree is agent-only and human read-only.
- SPEC is the requirement source.
- Goal Tree is derived from approved SPEC.
- Graph canonical path is `.agentflow/graph/`.
- Current Graph output compatibility path is `.agentflow/output/graph/`.
- AgentRun is not authorized yet.

## Allowed Actions

- Read project files.
- Read Graph status.
- Read Project File Reader metadata.
- Read Goal Tree snapshot.
- Read existing SPEC drafts / approvals when they exist.
- Ask human clarification questions.
- Produce Requirement Intake Results before SPEC Draft previews.
- Produce SPEC Draft previews in conversation.

## Forbidden Actions

- Do not write user source code.
- Do not execute project commands.
- Do not run tests.
- Do not create or edit Goal Tree directly.
- Do not write approved SPEC without human confirmation.
- Do not start AgentRun.
- Do not create PRs or remote issues.
- Do not use legacy workflow paths.

## Required Workflow

Conversation
→ Request triage
→ Requirement intake filter
→ SPEC Draft Preview
→ Human confirmation
→ Approved SPEC
→ Goal Tree materialization
→ Future AgentRun

## SPEC First Rule

Feature, refactor, cleanup, and unclear change requests must go through SPEC Draft Preview before any Goal Tree materialization.

Before SPEC Authoring, the Agent must produce a Requirement Intake Result.

Only `ready-for-openspec` may proceed to SPEC Draft Preview.

## Goal Tree Rule

Goal Tree is an agent-only derived fact source under `.agentflow/goal-tree/`. Humans can inspect it through Desktop, but Desktop cannot write it.

## Agent Roles

### 1. Intake Agent / 需求接待 Agent

Status: enabled.

Receives human input, classifies request type, runs requirement-intake-filter, asks clarification questions, and decides whether a request is ready for SPEC. It cannot write SPEC files, Goal Tree, source code, or execute commands.

### 2. Spec Planning Agent / 规格计划 Agent

Status: planned.

Produces SPEC Draft Preview from `ready-for-openspec` intake results, waits for human confirmation, and later materializes Goal Tree from Approved SPEC. It cannot bypass intake, skip confirmation, execute issues, write source code, or run tests.

### 3. Build Agent / 实现执行 Agent

Status: not authorized yet.

Future role for TDD-driven implementation from approved Goal Tree issues. It cannot run without Approved SPEC, a Goal Tree issue, and TDD evidence.

### 4. Release Agent / 发布交付 Agent

Status: not authorized yet.

Future role for commit, PR, review, changelog, release note, deploy, rollback, and release evidence. It cannot create remote PRs or deploy in the current stage.

### 5. Audit Agent / 代码审计 Agent

Status: not authorized yet.

Future role for checking SPEC alignment, boundary compliance, architecture impact, test coverage, legacy reintroduction, unauthorized execution, and evidence completeness. V1 only provides the audit manual skeleton.

## Execution Boundary

AgentRun is not authorized in this stage. Agents must stop before source writes, command execution, tests, PR creation, or remote issue creation.

## Validation Rule

Before any output or future write, the Agent must verify that AGENTS.md, Agentflow.md, skills-lock.json, requirement-intake-filter, boundary-check, and validation skills were read.

## Boundary

If the requested action is outside the current authorized stage, stop and ask for confirmation or wait for the next AgentFlow requirement.
"#
    )
}

pub fn skill_templates() -> [AgentSkillTemplate; 6] {
    [
        AgentSkillTemplate {
            name: "request-triage",
            relative_path: ".agentflow/define/agent/skills/request-triage/SKILL.md",
            content: REQUEST_TRIAGE_SKILL,
        },
        AgentSkillTemplate {
            name: "requirement-intake-filter",
            relative_path: ".agentflow/define/agent/skills/requirement-intake-filter/SKILL.md",
            content: REQUIREMENT_INTAKE_FILTER_SKILL,
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

const REQUIREMENT_INTAKE_FILTER_SKILL: &str = r#"# requirement-intake-filter

Version: v1

## Purpose

Act as AgentFlow's requirement gate before OpenSpec Authoring.

This skill turns human conversation into a structured Requirement Intake Result and decides whether the request may enter OpenSpec Draft Preview.

## Required Reading

- `<project-root>/AGENTS.md`
- `.agentflow/define/agent/Agentflow.md`
- `.agentflow/define/agent/skills-lock.json`
- `request-triage`
- `boundary-check`
- `validation`

## Input Sources

Prefer project context before asking questions:

- Human conversation
- Current Project Workspace state
- Graph status
- Project File Reader metadata
- Existing Goal Tree snapshot
- Existing OpenSpec drafts or approvals
- Agentflow.md
- skills-lock.json
- request-triage result

## Output Contract

Return a Requirement Intake Result. Do not output OpenSpec.

```json
{
  "version": "requirement-intake-filter.v1",
  "status": "needs-clarification",
  "requestType": "feature",
  "summary": "One-sentence requirement summary.",
  "knowns": [],
  "unknowns": [],
  "clarifyingQuestions": [],
  "scopeCandidates": [],
  "nonGoalCandidates": [],
  "acceptanceCriteriaCandidates": [],
  "boundaryRisks": [],
  "recommendedNextStep": "ask-clarifying-questions"
}
```

## Status Definitions

- `ready-for-openspec`: The goal, initial scope, non-goals, acceptance direction, and boundaries are clear enough for OpenSpec Draft Preview.
- `needs-clarification`: The request is likely valid, but key context is missing.
- `answer-only`: The input is a question or explanation request, not a requirement.
- `blocked-by-boundary`: The user asked to bypass current AgentFlow boundaries.
- `defer`: The request depends on a future capability that is not available in the current stage.

## Filtering Steps

1. Restate the user request in one sentence.
2. Classify request type.
3. Extract known facts.
4. Identify missing facts.
5. Identify scope candidates.
6. Identify non-goal candidates.
7. Draft acceptance criteria candidates.
8. Check AgentFlow boundaries.
9. Decide intake status.
10. Return Requirement Intake Result.

## Clarification Rules

- Ask at most 3 questions.
- Questions must be specific.
- Questions must serve OpenSpec readiness.
- Do not ask for information already available in project context.
- Do not over-clarify when the request is good enough for a draft preview.

## Boundary Checks

Check whether the request asks the Agent to:

- Write user source code.
- Execute commands.
- Write OpenSpec fact sources.
- Write Goal Tree.
- Skip approved OpenSpec.
- Start AgentRun.
- Create remote PRs, issues, or external objects.
- Touch legacy paths.
- Bypass AGENTS.md, Agentflow.md, or skills-lock.json.

If out of bounds, return `blocked-by-boundary` and explain the allowed replacement flow.

## Examples

### Ready for OpenSpec

Input: "Add a local project picker that only reads files and shows them in Desktop."

Result:

- status: `ready-for-openspec`
- recommendedNextStep: `generate-openspec-draft-preview`

### Needs Clarification

Input: "Make the project page better."

Result:

- status: `needs-clarification`
- clarifyingQuestions: ask which page, what user outcome, and what must not change.

### Answer Only

Input: "What is OpenSpec?"

Result:

- status: `answer-only`
- recommendedNextStep: `answer-in-conversation`

### Blocked by Boundary

Input: "Skip OpenSpec and write the Goal Tree now."

Result:

- status: `blocked-by-boundary`
- recommendedNextStep: `explain-boundary-and-stop`

## Non-goals

- Do not copy external prompt-optimizer text.
- Do not optimize prompts.
- Do not output OpenSpec.
- Do not write `.agentflow/spec/**`.
- Do not write Goal Tree.
- Do not start AgentRun.
- Do not execute commands.
- Do not write user source code.
"#;

const OPENSPEC_AUTHORING_SKILL: &str = r#"# openspec-authoring

Version: v1

## Purpose

Generate an OpenSpec Draft Preview only after requirement-intake-filter returns `ready-for-openspec`.

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

- Do not run before Requirement Intake Result status is `ready-for-openspec`.
- Without human confirmation, do not write `.agentflow/spec/**`.
- SPEC is the requirement source.
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

- SPEC objective -> Goal.objective
- SPEC scope / non-goals -> Goal.scope / Goal.nonGoals
- SPEC phases / design stages -> Milestone.stageGoal
- SPEC tasks -> Issue.goal
- SPEC acceptance criteria -> Issue.acceptanceCriteria
- SPEC constraints / boundaries -> Issue.boundary
- SPEC task dependencies -> Issue.dependencies
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

- Was AGENTS.md read?
- Was Agentflow.md read?
- Was skills-lock.json read?
- Is SPEC-first preserved?
- Did the Agent avoid erroneous Goal Tree writes?
- Did the Agent avoid unauthorized command execution?
- Are there unresolved confirmation questions?
- Should the Agent stop?
"#;
