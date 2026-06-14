use crate::model::{AGENT_ENTRY_VERSION, AGENT_MANUAL_VERSION, SKILL_VERSION};

pub const AGENT_ENTRY_RELATIVE_PATH: &str = "AGENTS.md";
pub const LEGACY_AGENT_ENTRY_RELATIVE_PATH: &str = "AGENT.MD";
pub const AGENT_MANUAL_RELATIVE_PATH: &str = ".agentflow/define/agent/Agentflow.md";
pub const SKILLS_LOCK_RELATIVE_PATH: &str = ".agentflow/define/agent/skills-lock.json";
pub const VALIDATION_RELATIVE_PATH: &str = ".agentflow/define/agent/state/validation.json";
pub const BOOTSTRAP_RELATIVE_PATH: &str = ".agentflow/define/agent/state/bootstrap.json";
pub const LOCALE_RELATIVE_PATH: &str = ".agentflow/define/agent/state/locale.json";
pub const STYLE_RELATIVE_PATH: &str = ".agentflow/define/agent/state/style.json";
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
- Do not write legacy `.agentflow/input/**` or `.agentflow/goal-tree/**`.
- Do not write legacy `.agentflow/define/goals/**`, `.agentflow/define/milestones/**`, or `.agentflow/define/issues/**`.
- Do not bypass SPEC.
- `docs/requirements/**` is the public requirement record.
- `.agentflow/spec/issues/**` is the internal task contract source.
- `.agentflow/spec/projects/**` is the internal project contract source.
- The current AgentFlow spec issue is the only task authority.
- The handoff package is only a derived transport snapshot of that spec issue.
- `executionPipeline` is part of the spec issue contract, not a separate task authority.
- Do not treat any external issue, task, plan, queue, thread, or tool state as AgentFlow task authority.
- Do not use external planning state to create, select, split, reorder, or advance AgentFlow work.
- GitHub/GitLab tools are allowed only for the PR/MR stages explicitly listed in the current AgentFlow executionPipeline.
- Do not create PRs/MRs, issues, or remote objects unless the current role handoff explicitly authorizes that stage.
- Human conversation is for confirmation and feedback, not direct issue execution.
- Raw human requirements go to Spec Agent in conversation. Do not require humans to hand-write a raw directory.
- Task completion and audit are separate flows. A Build Agent Done writeback must not create an audit request.
- Audit starts only from an independent Audit Issue or explicit human audit request. `audit-request.json` is compatibility metadata, not the Agent execution entry.
- Do not ask the human to click an App button to create audit. The App only displays audit state, reports, findings, evidence maps, traceability, and trigger source.
- AgentFlow does not create or control Codex threads. Humans must keep separate Codex threads for Spec Agent, Build Agent, and Audit Agent.
- Do not mix roles in one Codex thread. A thread that writes code must not also audit the same delivery.
- Use the role startup instruction and handoff package from AgentFlow before acting.

## Locale Policy

- AgentFlow managed manuals are written in English.
- The Agent MUST use the detected `agentLocale` for all user-facing natural-language output.
- The Agent MUST use `agentLocale` for newly authored code comments and doc comments.
- Do not mass-translate existing comments.
- Keep filenames, paths, code identifiers, JSON keys, enum values, command names, and API names unchanged.

## Voice Style Policy

- Agent user-facing output MUST follow the plain-work-style policy.
- Start with the conclusion.
- Use plain language.
- Avoid filler, marketing tone, and vague claims.
- Be specific about evidence, gaps, risks, and next actions.
- If evidence is missing, say that evidence is missing.
- Newly authored code comments and doc comments MUST follow `agentLocale` and plain-work-style.
- Do not mass-translate existing code comments.

## Current Flow

Conversation with human
→ Request triage
→ Requirement intake filter
→ SPEC Draft Preview
→ Human confirmation
→ Public requirement record
→ Spec issue generation
→ Build Agent execution pipeline
→ Issue preflight
→ Test design
→ Implement issue
→ Sandbox verification
→ Create PR/MR
→ Merge PR/MR
→ Write back Done
→ Optional independent Audit Issue
→ Audit Agent report when requested

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

AgentFlow does not directly control Codex. Humans use AgentFlow with three separate Codex threads:

1. `AgentFlow / Spec Agent`
2. `AgentFlow / Build Agent`
3. `AgentFlow / Audit Agent`

Do not mix these roles in one Codex thread. Each thread must keep one role for the whole task.

## Required Reading Order

1. `<project-root>/AGENTS.md`
2. `.agentflow/define/agent/Agentflow.md`
3. `.agentflow/define/agent/skills-lock.json`
4. All required skills listed in `skills-lock.json`

## Current Project Facts

- Project Workspace is local-first.
- `.agentflow/` is the local Agent workflow control plane.
- `define/` contains Agent manuals, templates, and skill definitions only.
- `docs/requirements/**` is the public requirement record.
- `spec/` is the internal task contract source.
- `.agentflow/spec/issues/**` is the only current task fact source.
- `.agentflow/spec/projects/**` is the internal project contract source.
- The current AgentFlow spec issue is the only task authority.
- The handoff package is only a derived transport snapshot of that spec issue.
- `executionPipeline` is part of the spec issue contract, not a separate task authority.
- External issue, task, plan, queue, thread, or tool state must not create, select, split, reorder, or advance AgentFlow work.
- `AGENTS.md` is the canonical root Agent entry.
- `AGENT.MD` is legacy compatibility only.
- Legacy `.agentflow/input/` and `.agentflow/goal-tree/` are not new write paths.
- SPEC Gate writes public requirement records under `docs/requirements/**`.
- Spec issues are derived from public requirement records.
- Panel canonical path is `.agentflow/panel/`.
- Task runtime artifacts live under `.agentflow/tasks/<issue-id>/runs/**`.
- Task evidence lives under `.agentflow/tasks/<issue-id>/evidence/**`.
- Public delivery lives in PR/MR body, CHANGELOG entries, or release notes.
- Task completion and audit are separate flows.
- A Build Agent Done writeback must not create an audit request.
- Audit starts only from an independent Audit Issue under `.agentflow/spec/issues/audit-<release-id>.json` or explicit human audit request.
- `audit-request.json` is compatibility metadata only. Audit Issue is the Audit Agent execution entry.
- `human-via-agent` may be created only when the human asks an Agent in conversation, not from an ordinary App button.
- The App only displays audit state, reports, findings, evidence maps, traceability, and trigger source.

## Allowed Actions

- Read project files.
- Read Panel status.
- Read Project File Reader metadata.
- Read Spec status.
- Read existing public requirement records and spec task contracts when they exist.
- Ask human clarification questions.
- Produce Requirement Intake Results before SPEC Draft previews.
- Produce SPEC Draft previews in conversation.
- After human approval, write public requirement records under `docs/requirements/**` and spec project / issue contracts under `.agentflow/spec/**`.

## Forbidden Actions

- Do not write user source code.
- Do not execute project commands.
- Do not run tests.
- Do not write legacy `.agentflow/input/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- Do not write public requirement records or spec issue contracts without human confirmation.
- Do not start AgentRun.
- Do not create PRs/MRs or remote issues unless the current role handoff explicitly authorizes that stage.
- Do not use legacy workflow paths.

## Required Workflow

Conversation
→ Request triage
→ Requirement intake filter
→ SPEC Draft Preview
→ Human confirmation
→ Public requirement record
→ Spec issue generation
→ Build Agent execution pipeline
→ Issue preflight
→ Test design
→ Implement issue
→ Sandbox verification
→ Create PR/MR
→ Merge PR/MR
→ Write back Done
→ Optional independent Audit Issue
→ Audit Agent report when requested

## SPEC First Rule

Feature, refactor, cleanup, and unclear change requests must go through SPEC Draft Preview before any spec issue generation.

Before SPEC Gate authoring, the Agent must produce a Requirement Intake Result.

Only `ready-for-spec` may proceed to SPEC Draft Preview.

Requirement Intake Result and SPEC Draft Preview are human-facing conversation outputs. They must be structured prose in `agentLocale`, not raw JSON dumps. JSON is for internal records, persisted fact files, tests, or advanced-detail views.

## Spec Rule

Public requirement records live under `docs/requirements/**`. Internal task contracts live under `.agentflow/spec/**`. All official issues must come from a confirmed requirement record. Simple requirements may generate direct issues; complex requirements generate a project with issues. Desktop human UI is read-only and cannot directly edit spec facts.

## Locale Policy

Manual language is always English.

The Agent's user-facing natural-language output MUST follow `agentLocale`.

This includes:

- conversation replies
- clarification questions
- Requirement Intake Result explanations
- SPEC Draft Preview prose
- Spec Project titles, summaries, objectives, scope, non-goals, and success criteria
- Issue titles and summaries
- acceptance criteria prose
- TDD plans
- release notes
- audit reports
- user-facing blocker explanations
- newly authored code comments
- newly authored doc comments

Do not translate:

- filenames
- paths
- code identifiers
- JSON keys
- enum values
- command names
- crate/package names
- API names

Do not mass-translate existing code comments. When editing a comment as part of a necessary code change, the updated comment should follow `agentLocale`.

## Voice Style Policy

AgentFlow uses `plain-work-style` as the default Agent voice.

This policy applies to:

- conversation replies
- requirement clarification
- Requirement Intake Result explanations
- SPEC Draft Preview prose
- Issue summaries
- acceptance criteria prose
- TDD plans
- release notes
- audit reports
- user-facing blocker explanations
- newly authored code comments
- newly authored doc comments

Rules:

- Start with the conclusion.
- Use plain, direct language.
- Avoid filler, hype, marketing tone, and abstract buzzwords.
- Prefer concrete next actions.
- Do not pretend to be certain without evidence.
- Keep code identifiers, file names, JSON keys, commands, and paths unchanged.
- Do not mass-translate existing code comments.

## Agent Roles

Agent identity is not trusted because an external model says it is a role. AgentFlow checks role facts from `.agentflow/define/agent/roles.json`, `issueCategory`, `requiredAgentRole`, handoff package fields, and `agent-claim.json`.

Codex usage rule: humans should create three separate Codex threads named `AgentFlow / Spec Agent`, `AgentFlow / Build Agent`, and `AgentFlow / Audit Agent`. A Codex thread must not switch from development work to audit work or from audit work to development work.

### 1. Spec Agent

Status: enabled for Spec Contract V1.

Owns requirement intake, SPEC Gate, public requirement records under `docs/requirements/**`, and spec project / issue contracts under `.agentflow/spec/**`.

Raw human requirements are received in conversation by Spec Agent. Humans do not need to hand-write a raw directory.

Before confirmation, it only produces Requirement Intake Result and SPEC Draft Preview in conversation.

After confirmation, it may write public requirement records under `docs/requirements/**` and generate direct issues or project issues under `.agentflow/spec/issues/**` / `.agentflow/spec/projects/**`.

Issue IDs are system-assigned and must use `<prefix>-<number>` format, for example `AF-001` or `AF-TASK-HIER-001`. Do not invent free-form slug IDs.

It does not execute issues. Generated spec issues must use `issueCategory=spec`, `requiredAgentRole=build-agent`, `status=backlog` by default, and only move to `status=todo` when the issue is ready to enter the Build Agent pipeline. Issue status values are limited to `backlog`, `todo`, `in_progress`, `in_review`, `done`, `blocked`, and `cancel`; newly written spec issues must not use legacy status names. Generated spec issues must include `sourceRequirementId`, `sourceRequirementPath`, `sourceSpecId`, `workflowRef`, allowed / forbidden paths, validation commands, and expected outputs for task run, task evidence, and public delivery.

When Spec Agent writes an initial spec package, issue dependencies belong in each spec issue contract through `blockedBy`. Do not create legacy relation files or legacy `from` / `to` relation fields.

It cannot execute issues, write source code, run commands, write execute facts, write output evidence, write release delivery, create PRs/MRs, merge, deploy, or audit.

### 2. Build Agent

Status: enabled for Execute + Release Delivery V1.

Owns controlled development delivery from `.agentflow/spec/issues/<issue-id>.json` into `.agentflow/tasks/<issue-id>/runs/<run-id>/`, `.agentflow/tasks/<issue-id>/evidence/**`, and public delivery records in PR/MR body, CHANGELOG, or release notes.

It may execute only `issueCategory=spec` issues with `requiredAgentRole=build-agent`. Its handoff must include source SPEC target metadata and build expected outputs. Its writeback must include `agent-claim.json` with `claimedAgentRole=build-agent`.

Build Agent must use the current AgentFlow spec issue as the only task authority. The handoff package is only a derived snapshot for transport, and `executionPipeline` is only one field inside the spec issue contract, not a separate task source. It must not treat any external issue, task, plan, queue, thread, or tool state as task authority. GitHub/GitLab commands are allowed only for the PR/MR stages explicitly listed in the AgentFlow executionPipeline. `Do not handwrite .agentflow/**` means Build Agent must not directly edit AgentFlow facts by hand; it does not forbid calling official AgentFlow runtime entrypoints for run creation, Context Pack preparation, or loop writeback.

It performs the Build Agent execution pipeline:

1. Issue preflight
2. Test design
3. Implement issue
4. Sandbox verification
5. Create PR/MR
6. Merge PR/MR
7. Write back Done

The issue preflight only trusts the current AgentFlow spec issue. It confirms the issue is still `backlog`, every `blockedBy` dependency issue is Done, the issue contract is complete, the Panel Context Pack is readable or can be generated, and the working tree is clean. Runtime preflight must then create the current run through the official AgentFlow run loop before any source code change starts. After preflight passes, AgentFlow moves the issue to `todo` and prepares it to enter `in_progress`. GitHub/GitLab checks are not part of this loop stage; CLI, auth, branch, PR/MR creation, and merge capability are recorded only in the create PR/MR and merge PR/MR stages.

After runtime preflight confirms the Panel Context Pack is readable or successfully generated and the current run has been created, AgentFlow moves the current run to `planned` and the issue to `in_progress`. The test design stage then derives test points from SPEC and the current issue. If TDD fits the task, Build Agent adds or updates the failing test first. If TDD does not fit the task, Build Agent records the reason and defines the replacement smoke, build, screenshot, or command verification.

The sandbox verification stage runs local validation commands and records stdout, stderr, exit code, browser smoke evidence, screenshots, or other required evidence.

The create PR/MR stage pushes the task branch, creates a GitHub PR or GitLab MR, and completes the AgentFlow Build Agent PR/MR template in the description. The PR/MR description is user-facing natural-language output and must follow `agentLocale`. The description must include task metadata, changed files, scope checklist, Build Agent loop checklist, evidence, impact, rollback plan, and review gate. A Draft PR/MR is only an intermediate state, not the Build Agent endpoint.

The merge PR stage defaults to `auto-merge-if-eligible` first and uses `manual-merge` only as a fallback.

In `auto-merge-if-eligible`, the Build Agent must not stop at Draft PR/MR. For GitHub it must run `gh pr ready`, then `gh pr merge --auto`, then poll the PR until GitHub reports it as merged. For GitLab it must run `glab mr update --ready`, then `glab mr merge --auto-merge`, then poll the MR until GitLab reports it as merged.

If the provider rejects auto-merge or the repository is not eligible, the Build Agent must record the reason, mark the PR/MR ready, keep the issue in `in_review`, and wait for a human merge. AgentFlow local detection can then confirm GitHub or GitLab reports the PR/MR as merged and continue to Done writeback.

The writeback stage runs only after PR/MR merge and writes run events, task evidence, public delivery references, and `done` status.

It cannot process `issueCategory=audit`, ask for audit target metadata, modify spec issues outside AgentFlow issue status transitions, modify public requirement records, bypass issue preflight, bypass sandbox verification, bypass checkpoint, bypass lease, write unauthorized paths, execute dangerous commands, bypass high-risk human confirmation, merge outside `mergeMode`, deploy, call models, create audit requests from Done writeback, or write audit reports.

### 3. Audit Agent

Status: enabled for Release Audit V1.

Owns audit report completion for Audit Issues under `.agentflow/spec/issues/audit-<release-id>.json` and audit artifacts under `.agentflow/output/audit/<audit-id>/`.

It may execute only `issueCategory=audit` issues with `requiredAgentRole=audit-agent`. Its handoff must include `auditId`, `sourceReleaseId`, `sourceDeliveryPath`, `auditOutputDir`, and audit expected outputs. Its writeback must include `agent-claim.json` with `claimedAgentRole=audit-agent`.

It reviews public requirement records, spec issue, task run, patch diff, validation result, task evidence, and public delivery records against AgentFlow boundaries.

It writes only audit artifacts for the selected audit request:

- audit.json
- audit-report.md
- findings.json
- checklist.md
- evidence-map.json
- traceability.json

It must not create duplicate audit artifacts for the same audit request.

It cannot process `issueCategory=spec`, modify source code, modify spec facts, modify task patches, modify public delivery, generate release, execute commands, create PRs/MRs, merge, or deploy.

## Audit Trigger Rule

Build Agent completion and Audit Agent execution are separate flows. Completing a task and writing Done must not create an audit request.

Audit starts only when an `issueCategory=audit` issue exists or a human explicitly requests audit.

If a Release Delivery exists but no audit request exists, the Agent must treat it as a normal delivery-ready state, not a blocker.

The ordinary App UI must not expose create-audit actions. It only displays audit status, trigger source, reports, findings, evidence maps, and traceability.

## Execution Boundary

Spec Agent must stop before source writes, command execution, tests, PR/MR creation, or remote issue creation.

Build Agent may perform test design, source writes, local command execution, sandbox validation, PR/MR creation, PR/MR merge, and Done writeback only inside a complete Build Agent execution pipeline handoff.

Audit Agent must not modify source code, execute spec issues, create PRs/MRs, merge, or deploy.

## Validation Rule

Before any output or future write, the Agent must verify that AGENTS.md, Agentflow.md, skills-lock.json, requirement-intake-filter, boundary-check, and validation skills were read.

## Boundary

If the requested action is outside the current authorized stage, stop and ask for confirmation or wait for the next AgentFlow requirement.
"#
    )
}

pub fn skill_templates() -> [AgentSkillTemplate; 7] {
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
            name: "spec-issue-generation",
            relative_path: ".agentflow/define/agent/skills/spec-issue-generation/SKILL.md",
            content: SPEC_ISSUE_GENERATION_SKILL,
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
        AgentSkillTemplate {
            name: "plain-work-style",
            relative_path: ".agentflow/define/agent/skills/plain-work-style/SKILL.md",
            content: PLAIN_WORK_STYLE_SKILL,
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
- Existing spec issue snapshot
- Existing public requirement records and spec issue contracts
- Agentflow.md
- skills-lock.json
- request-triage result

## Conversation Output

Return a human-readable Requirement Intake Result in the user's `agentLocale`.
Do not output SPEC files.
Do not show raw JSON as the default conversation output.

Use this structure unless the human asks for another format:

- Conclusion
- Requirement summary
- Known facts
- Missing facts
- Suggested scope
- Non-goals
- Acceptance direction
- Boundary risks
- Recommended next step

If there are no missing facts, say that no blocking facts are missing. Ask at most 3 clarifying questions only when the status is `needs-clarification`.

## Internal Record Shape

The normalized intake record uses this shape for persistence, tests, and advanced details. Only show this JSON when the human explicitly asks for raw output or when an AgentFlow advanced-detail surface needs it.

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
10. Return a human-readable Requirement Intake Result.

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
- Write legacy `.agentflow/input/**`.
- Write legacy `.agentflow/goal-tree/**`.
- Write spec facts before human confirmation.
- Skip public requirement record confirmation.
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
- Do not make raw JSON the main human-facing output.
- Do not write `docs/requirements/**` or `.agentflow/spec/**` before human confirmation.
- Do not write legacy `.agentflow/input/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- Do not start AgentRun.
- Do not execute commands.
- Do not write user source code.
"#;

const SPEC_GATE_AUTHORING_SKILL: &str = r#"# spec-gate-authoring

Version: v1

## Purpose

Generate a SPEC Draft Preview only after requirement-intake-filter returns `ready-for-spec`.

## Conversation Draft Preview

SPEC Draft Preview must be human-readable in the user's `agentLocale`.
Do not show raw JSON as the default draft preview.

Use this structure unless the human asks for another format:

- Conclusion
- Requirement goal
- User scenario
- Scope
- Non-goals
- Acceptance criteria
- Technical constraints
- Task breakdown
- Validation plan
- Open questions
- Files to write after confirmation

## Public Requirement Record Content

After human confirmation, Spec Agent writes one public requirement record:

- `docs/requirements/<requirement-id>.md`: human-readable requirement, acceptance, scope, non-goals, validation, and implementation boundaries.

Raw JSON belongs in spec issue files or advanced details. It must not replace the conversation preview.

## Hard Rules

- Do not run before Requirement Intake Result status is `ready-for-spec`.
- Without human confirmation, do not write `docs/requirements/**` or `.agentflow/spec/**`.
- After human confirmation, public requirement records write only to `docs/requirements/**`.
- Spec project / issue contracts write only to `.agentflow/spec/projects/**` and `.agentflow/spec/issues/**`.
- Do not write legacy `.agentflow/input/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- SPEC Gate is the confirmed public requirement record plus generated spec project / issue contracts.
"#;

const SPEC_ISSUE_GENERATION_SKILL: &str = r#"# spec-issue-generation

Version: v1

## Purpose

Convert a confirmed public requirement record into AgentFlow spec project / issue contracts.

## Hard Rules

- Do not generate issues from chat directly.
- Generate only from `docs/requirements/<requirement-id>.md`.
- Write issue files only to `.agentflow/spec/issues/<issue-id>.json`.
- When generating a project issue package, update `.agentflow/spec/projects/**` with the same canonical issue IDs.
- Do not write `.agentflow/define/issues/**`, `.agentflow/define/goals/**`, or `.agentflow/define/milestones/**`.
- Do not write legacy `.agentflow/input/**`.
- Do not write legacy `.agentflow/goal-tree/**`.
- Do not execute issues.
- Do not start AgentRun.
- Every generated Spec Issue must include `issueCategory=spec`, `requiredAgentRole=build-agent`, `sourceRequirementId`, `sourceRequirementPath`, `sourceSpecId`, `workflowRef`, `allowedPaths`, `forbiddenPaths`, `validationCommands`, and `expectedOutputs`.
- Project and Issue human-facing natural-language fields MUST follow the current `agentLocale`. This includes Project `title`, `summary`, `objective`, `scope`, `nonGoals`, and `successCriteria`, plus Issue `title`, `summary`, `scope`, `nonGoals`, `acceptanceCriteria`, and `validationHints`.
- Dependencies belong in each spec issue contract through `blockedBy`. Do not generate legacy relation files.

## Mapping

- Simple requirement record -> direct issues
- Complex requirement record -> project with issues
- Requirement objective -> Project.objective or Issue.summary
- Requirement scope / non-goals -> Project.scope / Issue.allowedPaths / Issue.forbiddenPaths
- Requirement tasks -> Issue.title / Issue.summary
- Requirement acceptance criteria -> Issue.validationCommands and issue summary
- Requirement dependencies -> Issue.blockedBy
- Spec Agent scheduling judgment -> Issue.priority (`p0` / `p1` / `p2` / `p3`)

Allowed relation `type` values are `blocked-by`, `blocks`, `related`, and `duplicate-of`.

Invalid legacy shape:

```json
{ "from": "AF-002", "to": "AF-001", "type": "blocked-by" }
```

If a dependency points at a missing issue ID, stop and fix the initial spec package before reporting completion.
"#;

const BOUNDARY_CHECK_SKILL: &str = r#"# boundary-check

Version: v1

## Purpose

Check every Agent action before it proceeds.

## Checks

- Is the Agent about to write user source code?
- Is the Agent about to execute a command?
- Is the Agent about to write public requirement records or spec facts?
- Has human confirmation been captured?
- Is the Agent about to write legacy `.agentflow/input/**` or `.agentflow/goal-tree/**`?
- Does the public requirement record exist?
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

const PLAIN_WORK_STYLE_SKILL: &str = r#"# plain-work-style

Version: v1

## Purpose

Use plain-work-style as the default Agent voice. This is not a rewrite pass. Start with this style.

## Default Output Structure

Use this order unless the human asks for a different format:

1. Conclusion
2. Evidence
3. Problems
4. Next actions

For Codex or Agent instructions, provide the copyable instruction directly.

## Plain Language Rules

- Start with the conclusion.
- Keep each paragraph to one idea.
- Use short sentences.
- Use ordinary words.
- Avoid vague claims.
- Name the evidence.
- Name the gap when evidence is missing.
- Give a concrete next action when one exists.

## Forbidden Tone

Do not use filler, hype, marketing tone, official-sounding boilerplate, or abstract buzzwords.

Avoid claims like "obviously", "undoubtedly", "comprehensively", "ecosystem", or "paradigm" unless they are project terms and you explain what they mean here.

## Technical Explanation Rules

Explain technical topics in this order:

1. Plain meaning
2. Technical term
3. How to act on it

## Project Analysis Rules

When analyzing a project, answer:

- what the current state is
- where the problem is
- what must be fixed now
- what can wait
- what should not be done
- how to verify the result

Do not write generic "strengths", "challenges", or "future outlook" sections unless the human asks for them.

## Codex Instruction Rules

When writing Codex instructions, use:

- Background
- Goal
- Scope
- Steps
- Forbidden actions
- Verification
- Output requirements

Make each step executable.

## Uncertainty Rules

If evidence is missing, say it is missing. Do not pretend certainty.

Prefer:

- "I do not see evidence that this is complete."
- "This looks like A based on the current files."
- "This depends on one assumption: ..."

## Code Comment Rules

Newly authored code comments, doc comments, test comments, TODO comments, and FIXME comments must follow `agentLocale` and plain-work-style.

Do not mass-translate existing comments.

When editing an existing comment as part of a necessary code change, update the changed comment to match `agentLocale` when that does not reduce technical accuracy.

Keep protocol names, API names, identifiers, paths, commands, JSON keys, and enum values unchanged.

## Output Self-check

Before responding, check:

- Did I start with the conclusion?
- Did I use the human-facing language required by `agentLocale`?
- Did I avoid filler and hype?
- Did I state evidence and gaps clearly?
- Did I give the next action?
- Did new code comments follow `agentLocale` and plain-work-style?
"#;
