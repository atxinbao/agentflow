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
- Before producing a SPEC Draft Preview, every Agent MUST run the requirement-intake-filter skill.
- Do not write legacy `.agentflow/spec/**` or `.agentflow/goal-tree/**`.
- Do not bypass SPEC.
- Do not create PRs, issues, or remote objects unless explicitly authorized.
- Human conversation is for confirmation and feedback, not direct issue execution.

## Current Flow

Conversation with human
→ Request triage
→ Requirement intake filter
→ SPEC Draft Preview
→ Human confirmation
→ Approved SPEC
→ Input issue generation
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
- `input/` is the canonical requirement fact source.
- `AGENTS.md` is the canonical root Agent entry.
- `AGENT.MD` is legacy compatibility only.
- Legacy `.agentflow/spec/` and `.agentflow/goal-tree/` are not new write paths.
- SPEC Gate uses `product.md`, `tech.md`, and `approval.json` under `.agentflow/input/specs/`.
- Input issues are derived from Approved SPEC.
- Panel canonical path is `.agentflow/panel/`.
- AgentRun is not authorized yet.

## Allowed Actions

- Read project files.
- Read Panel status.
- Read Project File Reader metadata.
- Read Input status.
- Read existing input SPEC drafts / approvals when they exist.
- Ask human clarification questions.
- Produce Requirement Intake Results before SPEC Draft previews.
- Produce SPEC Draft previews in conversation.
- After human approval, write Approved SPEC and direct issues or project issues under `.agentflow/input/**`.

## Forbidden Actions

- Do not write user source code.
- Do not execute project commands.
- Do not run tests.
- Do not write legacy `.agentflow/spec/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- Do not write Approved SPEC without human confirmation.
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
→ Input issue generation
→ Future AgentRun

## SPEC First Rule

Feature, refactor, cleanup, and unclear change requests must go through SPEC Draft Preview before any input issue generation.

Before SPEC Gate authoring, the Agent must produce a Requirement Intake Result.

Only `ready-for-spec` may proceed to SPEC Draft Preview.

## Input Rule

Input is the canonical requirement fact source under `.agentflow/input/`. All official issues must come from Approved SPEC. Simple requirements generate direct issues; complex requirements generate a project with issues. Desktop human UI is read-only and cannot directly edit input facts.

## Agent Roles

### 1. Spec Agent / 需求规格 Agent

Status: enabled for Input Model V1.

Owns requirement intake, SPEC Gate, Approved SPEC, direct issues, and project issues under `.agentflow/input/**`.

After confirmation, it may write Approved SPEC and generate direct issues or project issues under `.agentflow/input/**`.

It cannot execute issues, write source code, run commands, write output evidence, write release delivery, create PRs, merge, deploy, or audit.

### 2. Build Agent / 实现交付 Agent

Status: enabled for Execute + Release Delivery V1.

Owns controlled development delivery from `.agentflow/input/issues/<issue-id>.json` into `.agentflow/execute/runs/<run-id>/`, `.agentflow/output/evidence/<run-id>.json`, and `.agentflow/output/release/<run-id>/`.

It performs preflight, lease, plan, checkpoint, patch, command record, validation, result, evidence, PR draft, PR metadata, review material, changelog, release note, and delivery record.

It cannot modify input issues, modify Approved SPEC, bypass preflight, bypass checkpoint, bypass lease, write unauthorized paths, execute dangerous commands, bypass high-risk human confirmation, merge, deploy, call models, or write audit reports.

### 3. Audit Agent / 代码审计 Agent

Status: not authorized yet.

Future role for reviewing Approved SPEC, input issue, execute run, patch diff, validation result, output evidence, and release delivery artifacts against AgentFlow boundaries.

It cannot modify source code, modify input facts, modify execute patches, modify release delivery, execute commands, create PRs, merge, or deploy.

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
            name: "spec-gate-authoring",
            relative_path: ".agentflow/define/agent/skills/spec-gate-authoring/SKILL.md",
            content: SPEC_GATE_AUTHORING_SKILL,
        },
        AgentSkillTemplate {
            name: "input-issue-generation",
            relative_path: ".agentflow/define/agent/skills/input-issue-generation/SKILL.md",
            content: INPUT_ISSUE_GENERATION_SKILL,
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

- feature: must enter SPEC Gate authoring.
- unclear change: ask questions first; do not write fact sources.
- bug: require reproduction information, current behavior, and expected behavior.
- cleanup: constrain scope and non-goals before planning.
- question: answer only; do not write fact sources.
- research: output findings only unless the human confirms entry into SPEC Gate.
"#;

const REQUIREMENT_INTAKE_FILTER_SKILL: &str = r#"# requirement-intake-filter

Version: v1

## Purpose

Act as AgentFlow's requirement gate before SPEC Gate authoring.

This skill turns human conversation into a structured Requirement Intake Result and decides whether the request may enter SPEC Draft Preview.

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
- Panel status
- Project File Reader metadata
- Existing input issue snapshot
- Existing input SPEC drafts or approvals
- Agentflow.md
- skills-lock.json
- request-triage result

## Output Contract

Return a Requirement Intake Result. Do not output SPEC files.

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

- `ready-for-spec`: The goal, initial scope, non-goals, acceptance direction, and boundaries are clear enough for SPEC Draft Preview.
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
- Questions must serve SPEC readiness.
- Do not ask for information already available in project context.
- Do not over-clarify when the request is good enough for a draft preview.

## Boundary Checks

Check whether the request asks the Agent to:

- Write user source code.
- Execute commands.
- Write legacy `.agentflow/spec/**`.
- Write legacy `.agentflow/goal-tree/**`.
- Write input facts before human confirmation.
- Skip Approved SPEC.
- Start AgentRun.
- Create remote PRs, issues, or external objects.
- Touch retired runtime paths.
- Bypass AGENTS.md, Agentflow.md, or skills-lock.json.

If out of bounds, return `blocked-by-boundary` and explain the allowed replacement flow.

## Examples

### Ready for SPEC Draft Preview

Input: "Add a local project picker that only reads files and shows them in Desktop."

Result:

- status: `ready-for-spec`
- recommendedNextStep: `generate-spec-draft-preview`

### Needs Clarification

Input: "Make the project page better."

Result:

- status: `needs-clarification`
- clarifyingQuestions: ask which page, what user outcome, and what must not change.

### Answer Only

Input: "What is SPEC Gate?"

Result:

- status: `answer-only`
- recommendedNextStep: `answer-in-conversation`

### Blocked by Boundary

Input: "Skip SPEC Gate and write issues now."

Result:

- status: `blocked-by-boundary`
- recommendedNextStep: `explain-boundary-and-stop`

## Non-goals

- Do not copy external prompt-optimizer text.
- Do not optimize prompts.
- Do not output SPEC files.
- Do not write `.agentflow/input/**` before human confirmation.
- Do not write legacy `.agentflow/spec/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- Do not start AgentRun.
- Do not execute commands.
- Do not write user source code.
"#;

const SPEC_GATE_AUTHORING_SKILL: &str = r#"# spec-gate-authoring

Version: v1

## Purpose

Generate a SPEC Draft Preview only after requirement-intake-filter returns `ready-for-spec`.

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
- Product draft for future `product.md`
- Tech draft for future `tech.md`
- Tasks draft
- Validation plan

## Hard Rules

- Do not run before Requirement Intake Result status is `ready-for-spec`.
- Without human confirmation, do not write `.agentflow/input/**`.
- Do not write legacy `.agentflow/spec/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- SPEC Gate is `product.md` + `tech.md` + `approval.json`.
"#;

const INPUT_ISSUE_GENERATION_SKILL: &str = r#"# input-issue-generation

Version: v1

## Purpose

Convert Approved SPEC into AgentFlow input issues.

## Hard Rules

- Do not generate issues from chat directly.
- Generate only from Approved SPEC with `product.md`, `tech.md`, and `approval.json`.
- Write only `.agentflow/input/**`.
- Do not write legacy `.agentflow/spec/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- Do not execute issues.
- Do not start AgentRun.

## Mapping

- Simple Approved SPEC -> direct issues
- Complex Approved SPEC -> project with issues
- SPEC objective -> Project.objective or Issue.summary
- SPEC scope / non-goals -> Project.scope / Issue.scope / Issue.nonGoals
- SPEC tasks -> Issue.title / Issue.summary
- SPEC acceptance criteria -> Issue.acceptanceCriteria
- SPEC validation plan -> Issue.validationHints
- SPEC dependencies -> Issue.relations
- Spec Agent judgment -> Issue.riskLevel
"#;

const BOUNDARY_CHECK_SKILL: &str = r#"# boundary-check

Version: v1

## Purpose

Check every Agent action before it proceeds.

## Checks

- Is the Agent about to write user source code?
- Is the Agent about to execute a command?
- Is the Agent about to write input facts?
- Has human confirmation been captured?
- Is the Agent about to write legacy `.agentflow/spec/**` or `.agentflow/goal-tree/**`?
- Does Approved SPEC exist?
- Is the Agent about to start AgentRun?
- Is the Agent about to create a remote object?
- Does the action touch retired runtime paths?

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
- Did the Agent avoid erroneous legacy SPEC / Goal Tree writes?
- Did the Agent avoid unauthorized command execution?
- Are there unresolved confirmation questions?
- Should the Agent stop?
"#;
