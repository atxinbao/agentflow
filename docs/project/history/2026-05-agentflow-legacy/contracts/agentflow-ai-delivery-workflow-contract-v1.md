# AgentFlow AI Delivery Workflow Contract v1

创建日期：2026-05-26
更新日期：2026-05-26
执行者：Codex
Status: Ready for ARC Review
Owner: `@003 / PRD`
Review: `@005 / ARC`
Implementation: `@000 / AIE`
Scope: AgentFlow MVP Workflow Core

## 0. Contract Purpose

This document is the canonical MVP workflow contract for AgentFlow.

It is intended to be directly usable by:

- `@003 / PRD` to confirm product behavior and non-goals.
- `@005 / ARC` to review state machines, data boundaries, race conditions, and auditability.
- `@000 / AIE` to create issue contracts and implement the workflow core.

This contract is not a UI design document and not a Linear clone spec.

## 1. Product Thesis

AgentFlow is not a Linear issue runner.

AgentFlow is a controlled delivery system for AI coding agents.

Its job is to make AI-generated software work:

```text
scoped
sequenced
leased
reviewable
testable
auditable
documented
```

The core operating model is:

```text
Human controls scope
Milestone controls phase
Issue controls execution
Eligibility controls when
Lease controls who
PR / checks / evidence controls Done
Audit / docs controls closure
```

## 2. MVP Closed Loop

AgentFlow MVP must support this closed loop:

```text
Workspace / Team
-> Project
-> Milestone
-> Issue
-> Eligibility
-> Lease
-> Execution Run
-> PR / Checks
-> Evidence
-> Issue Done
-> Milestone Review
-> Project Audit
-> Root Docs Refresh
-> Project Final Evidence
```

The system must make it difficult for an AI agent to:

```text
work outside human-confirmed scope
skip dependencies
run ahead of the active milestone
work on multiple code-changing issues at once
claim Done without merge / checks / evidence
finish a milestone without review
finish a project without audit / docs refresh
```

## 3. MVP Non-Goals

AgentFlow v1 must not attempt to be:

```text
a full Linear clone
a full GitHub replacement
a general project management tool
a multi-agent swarm planner
a UI-first workflow board
a generic task runner
a chat-only coding assistant
a cloud sync service
an account / payment system
an overseas AI service proxy
```

UI may exist later, but the MVP must first stabilize workflow contracts, state machines, eligibility computation, lease behavior, evidence standards, and closure gates.

## 4. Hard Invariants

These rules are binding for MVP v1.

```text
1. Agent executes only Issues.
2. Project scope is human-confirmed.
3. Only one Milestone is Active per Project.
4. Only one code-changing Issue may be Eligible / Leased / InProgress per Project.
5. Ready is not Eligible.
6. Eligible is computed, not manually asserted.
7. Leased is not Done.
8. Every code-changing Execution Run must be tied to a Lease.
9. Every execution attempt creates an Execution Run.
10. Every merged PR or merge-equivalent must be tied to an Issue.
11. Done requires merge-equivalent + required checks + evidence.
12. Milestone Done requires Milestone Review.
13. Project Done requires Code Audit + Root Docs Refresh + Final Evidence.
14. Agent cannot silently expand scope.
15. Human gates cannot be bypassed by CLI or Desktop.
16. Desktop MVP remains read-only for run / verify / review / merge.
17. Roadmap candidate is not execution authorization.
18. IssueContract is the execution authorization boundary.
```

## 5. Local-First Boundary

AgentFlow MVP is local-first.

Current v1 local behavior:

```text
PR / Checks may be represented by local evidence or future-provider placeholders.
No remote PR is created automatically.
No remote issue is created automatically.
No CI system is required.
No cloud sync is required.
No model call is required for workflow state transitions.
```

Future provider adapters may map local records to GitHub, GitLab, Linear, or CI systems, but the local contract remains the source of truth.

## 6. Core Entity Model

### 6.1 Workspace

Purpose:

```text
The top-level local container for teams, projects, shared policies, and agent limits.
```

Required fields:

```yaml
workspaceId: string
name: string
status: active | archived
members:
  - memberId: string
    displayName: string
    role: owner | maintainer | viewer
agentPolicies:
  maxActiveMilestonesPerProject: 1
  maxActiveCodeChangingIssuesPerProject: 1
  requireHumanProjectConfirmation: true
  requireMilestoneReviewGate: true
  requireProjectAuditGate: true
  defaultLeaseTtlMinutes: number
createdAt: string
updatedAt: string
```

Rules:

- Workspace owns Projects and Teams.
- Workspace policy provides default limits, but Project may add stricter rules.
- Workspace does not execute work.

### 6.2 Team

Purpose:

```text
A grouping boundary for Projects and Issues.
```

Required fields:

```yaml
teamId: string
workspaceId: string
name: string
status: active | archived
projectIds: string[]
issueIds: string[]
createdAt: string
updatedAt: string
```

Rules:

- Team may contain multiple Projects.
- Issue may be shown under a Team through `projectLink.teamId`.
- Team does not override Project / Milestone execution gates.

### 6.3 Project

Purpose:

```text
A human-confirmed product goal and execution boundary.
```

Required fields:

```yaml
projectId: string
workspaceId: string
teamIds: string[]
title: string
status: draft | confirmed | active | audit | docs-refresh | final-review | done | blocked | paused | cancelled | failed
owner: string
humanApprover: string
goal: string
nonGoals: string[]
scopeBoundaries:
  allowedAreas: string[]
  blockedAreas: string[]
  allowedCommands: string[]
  blockedCommands: string[]
successCriteria: string[]
riskLevel: low | medium | high
repo:
  root: string
  provider: local | github | gitlab | other
  remoteUrl: string | null
  baseBranch: string
milestoneIds: string[]
activeMilestoneId: string | null
humanGates:
  projectConfirmation: boolean
  milestoneReview: boolean
  finalApproval: boolean
createdAt: string
updatedAt: string
```

Rules:

- Project cannot become `confirmed` without human approval.
- Project cannot become `active` without at least one `ready` Milestone.
- Project cannot enter `audit` until all Milestones are `done`.
- Project cannot become `done` until Code Audit, Root Docs Refresh, Final Evidence Summary, and Human Final Approval are complete.

### 6.4 Milestone

Purpose:

```text
A phase gate inside a Project.
```

Required fields:

```yaml
milestoneId: string
projectId: string
title: string
status: draft | ready | active | review | done | blocked | paused | cancelled | failed
goal: string
nonGoals: string[]
gateCriteria: string[]
dependencyMilestoneIds: string[]
issueIds: string[]
completedIssueIds: string[]
reviewRequired: true
reviewId: string | null
humanApprovalRequired: true
createdAt: string
updatedAt: string
```

Rules:

- Only one Milestone may be `active` per Project.
- Milestone cannot become `ready` without gate criteria.
- Milestone cannot become `active` if dependency Milestones are not `done`.
- Milestone cannot become `done` without Milestone Review.
- Next Milestone cannot become `active` until previous Milestone is `done`.

### 6.5 Issue

Purpose:

```text
The only execution atom for an agent.
```

Required fields:

```yaml
issueId: string
projectId: string
milestoneId: string
teamId: string
title: string
status: draft | ready | eligible | leased | in-progress | pr | checks-passing | merged | evidence-captured | done | blocked | failed | cancelled | needs-human-review
kind: code-changing | docs-only | test-only | read-only-investigation | evidence-summary | milestone-review
goal: string
nonGoals: string[]
dependencies: string[]
acceptanceCriteria: string[]
expectedFiles: string[]
blockedFiles: string[]
riskLevel: low | medium | high
testPlan: string[]
evidenceRequired: string[]
rollbackPlan: string
projectLink:
  teamId: string
  projectId: string
  milestoneId: string
  linkSource: string
humanGate:
  highRiskApproval: boolean
  scopeExpansionApproval: boolean
  blockedFileApproval: boolean
createdAt: string
updatedAt: string
```

Rules:

- Issue cannot become `ready` unless acceptance criteria, test plan, risk level, expected files, evidence requirements, and rollback plan are present.
- Issue cannot become `eligible` unless Eligibility Engine passes.
- Issue cannot become `leased` without an active Lease.
- Issue cannot become `done` without required evidence.
- Agent may propose scope changes, but Human must approve them.

### 6.6 Eligibility Snapshot

Purpose:

```text
The computed answer to: can this Issue execute now?
```

Required fields:

```yaml
eligibilityId: string
issueId: string
projectId: string
milestoneId: string
eligible: boolean
computedAt: string
computedBy: eligibility-engine
reasonsPassed: string[]
reasonsFailed: string[]
warnings: string[]
riskLevel: low | medium | high
requiresHumanApproval: boolean
singleCodeChangingIssueSatisfied: boolean
repoState:
  clean: boolean | unknown
  baseLatest: boolean | unknown
  conflictingPrOpen: boolean | unknown
```

Rules:

- Eligibility is computed, not manually assigned.
- Eligibility Snapshot may be stored as evidence, but must be recomputable.
- A stale Eligibility Snapshot cannot authorize execution after Project, Milestone, Issue, dependency, repo, or lease state changes.

### 6.7 Lease

Purpose:

```text
Prevents multiple agents from working on the same code-changing Issue.
```

Required fields:

```yaml
leaseId: string
issueId: string
projectId: string
milestoneId: string
ownerAgentId: string
status: active | renewed | released | expired | recovered | cancelled
leaseScope: code-changing | non-code-changing | read-only
leasedAt: string
expiresAt: string
renewedAt: string | null
releasedAt: string | null
heartbeatAt: string | null
staleRecoveryReason: string | null
createdAt: string
updatedAt: string
```

Rules:

- Agent cannot start an Execution Run without a valid Lease.
- Code-changing Lease is exclusive per Project.
- Lease does not authorize scope expansion.
- Stale lease recovery must create an event with reason.

### 6.8 Execution Run

Purpose:

```text
Records one concrete attempt by an agent to execute an Issue.
```

Required fields:

```yaml
runId: string
issueId: string
leaseId: string
agentId: string
status: planned | running | completed | failed | cancelled | blocked
startedAt: string
endedAt: string | null
baseCommit: string | null
headCommit: string | null
executionPlan:
  goal: string
  nonGoals: string[]
  plannedSteps: string[]
  expectedFiles: string[]
  blockedFiles: string[]
  testPlan: string[]
  rollbackPlan: string
changedFiles: string[]
commandsRun:
  - command: string
    status: passed | failed | skipped
    exitCode: number | null
    outputPath: string | null
testResults:
  status: passed | failed | skipped | missing
  summary: string
failureReason: string | null
createdAt: string
updatedAt: string
```

Rules:

- Every code-changing agent attempt creates an Execution Run.
- Execution Run must stop if scope expands beyond the Issue.
- Execution Run must stop if blocked files need modification without approval.
- Execution Run must stop if test plan is impossible.
- Product Feature Controlled Run v0 maps this contract to local dry-run only: `AgentRun.runPlan` records goal, non-goals, expected files, blocked files / areas, planned steps, validation commands, evidence requirements, and rollback plan before any real code execution exists.

### 6.9 Pull Request / Merge Evidence

Purpose:

```text
Represents the review and merge boundary.
```

Required fields:

```yaml
prId: string
issueId: string
runId: string
provider: local | github | gitlab | other
repo: string
branch: string | null
url: string | null
status: not-created | opened | reviewed | merged | closed | local-merge-equivalent
baseCommit: string | null
headCommit: string | null
checks:
  - name: string
    status: queued | running | passed | failed | skipped | missing
    url: string | null
    completedAt: string | null
reviewStatus: not-required | pending | approved | changes-requested
mergedAt: string | null
mergeCommit: string | null
createdAt: string
updatedAt: string
```

Rules:

- MVP local mode may use `local-merge-equivalent`.
- Future Git provider adapter must map remote PRs into this record.
- Issue cannot become `done` without PR / merge-equivalent evidence.

### 6.10 Evidence

Purpose:

```text
Proves that the Issue was completed correctly.
```

Required fields:

```yaml
evidenceId: string
issueId: string
runId: string
prId: string | null
commitHash: string | null
changedFiles: string[]
checksSummary:
  status: passed | failed | skipped | missing
  requiredChecks: string[]
  passedChecks: string[]
  failedChecks: string[]
testOutput:
  summary: string
  outputPaths: string[]
acceptanceCriteriaCoverage:
  - criterion: string
    status: passed | failed | deferred
    evidence: string
manualVerification: string[]
screenshots: string[]
logs: string[]
rollbackPlan: string
knownGaps: string[]
createdAt: string
```

Rules:

- No evidence, no Done.
- Evidence must explicitly map acceptance criteria to proof.
- Evidence may reference logs or screenshots, but must not rely on chat-only claims.

### 6.11 Milestone Review

Purpose:

```text
Summarizes evidence for all Issues in a Milestone and gates progression.
```

Required fields:

```yaml
milestoneReviewId: string
milestoneId: string
projectId: string
status: draft | generated | human-approved | rejected
completedIssues:
  - issueId: string
    status: done
    evidenceId: string
    prId: string | null
prSummary: string
checksSummary: string
acceptanceCoverage: string
behaviorChanges: string[]
configChanges: string[]
schemaChanges: string[]
apiChanges: string[]
risks: string[]
deferredWork: string[]
recommendation: proceed | hold
humanApproval:
  required: true
  approver: string | null
  approvedAt: string | null
createdAt: string
updatedAt: string
```

Rules:

- Milestone Review is a gate, not only a summary.
- Next Milestone cannot activate until current review is complete.
- MVP may generate the review locally, but human approval remains a gate.

### 6.12 Project Closure

Purpose:

```text
Closes the entire Project after all Milestones are Done.
```

Required fields:

```yaml
projectClosureId: string
projectId: string
status: audit | docs-refresh | final-review | done | blocked
codeAudit:
  status: not-started | running | passed | failed | accepted-risk
  findings: string[]
  requiredFixes: string[]
  acceptedRisks: string[]
docsRefresh:
  status: not-started | running | passed | failed
  updatedDocs: string[]
  unchangedDocs: string[]
  missingDocs: string[]
  notes: string[]
finalEvidenceSummary:
  projectGoal: string
  milestonesCompleted: string[]
  issuesCompleted: string[]
  prsMerged: string[]
  checksPassed: string[]
  behaviorChanges: string[]
  docsUpdated: string[]
  knownRisks: string[]
  deferredWork: string[]
  recommendation: approve | hold
humanFinalApproval:
  required: true
  approver: string | null
  approvedAt: string | null
createdAt: string
updatedAt: string
```

Rules:

- Project cannot become `done` until Code Audit, Root Docs Refresh, Final Evidence Summary, and Human Final Approval are complete.

## 7. State Machines

### 7.1 Project State Machine

States:

```text
draft
confirmed
active
audit
docs-refresh
final-review
done
blocked
paused
cancelled
failed
```

Allowed transitions:

| From | To | Guard |
| --- | --- | --- |
| draft | confirmed | Human confirms goal, non-goals, scope boundaries, repo, base branch, success criteria |
| confirmed | active | At least one Milestone is ready |
| active | audit | All Milestones are done |
| audit | docs-refresh | Code Audit is complete |
| docs-refresh | final-review | Root Docs Refresh is complete |
| final-review | done | Human final approval is complete |
| draft / confirmed / active | blocked | Blocking dependency or human gate exists |
| blocked | previous non-terminal state | Blocker resolved and event recorded |
| any non-terminal | paused | Human pauses Project |
| paused | previous non-terminal state | Human resumes Project |
| any non-terminal | cancelled | Human cancels Project |
| any non-terminal | failed | Unrecoverable execution or contract failure |

Forbidden transitions:

```text
draft -> active
active -> done
audit -> done
docs-refresh -> done
blocked -> done
cancelled -> active
failed -> active
```

### 7.2 Milestone State Machine

States:

```text
draft
ready
active
review
done
blocked
paused
cancelled
failed
```

Allowed transitions:

| From | To | Guard |
| --- | --- | --- |
| draft | ready | goal, non-goals, gate criteria, issue list or issue generation plan exist |
| ready | active | Project active, dependencies done, no other active Milestone in Project |
| active | review | all Milestone Issues are done |
| review | done | Milestone Review generated and human-approved |
| draft / ready / active | blocked | blocker recorded |
| blocked | previous non-terminal state | blocker resolved and event recorded |
| any non-terminal | paused | Human pauses Milestone |
| paused | previous non-terminal state | Human resumes Milestone |
| any non-terminal | cancelled | Human cancels Milestone |
| any non-terminal | failed | unrecoverable Milestone failure |

Forbidden transitions:

```text
draft -> active
ready -> done
active -> done
review -> active without explicit rejection / repair event
done -> active
```

### 7.3 Issue State Machine

States:

```text
draft
ready
eligible
leased
in-progress
pr
checks-passing
merged
evidence-captured
done
blocked
failed
cancelled
needs-human-review
```

Allowed transitions:

| From | To | Guard |
| --- | --- | --- |
| draft | ready | Issue completeness gate passes |
| ready | eligible | Eligibility Engine returns eligible true |
| eligible | leased | Lease acquired |
| leased | in-progress | Execution Run started |
| in-progress | pr | PR / local merge-equivalent record opened |
| pr | checks-passing | required checks pass |
| checks-passing | merged | PR merged or local merge-equivalent completed |
| merged | evidence-captured | evidence complete |
| evidence-captured | done | review / update complete |
| ready / eligible / leased / in-progress | blocked | blocker recorded |
| in-progress | needs-human-review | scope expansion, blocked file, high-risk path, missing dependency, impossible test plan |
| in-progress | failed | run failed and no automatic repair path exists |
| blocked | ready | blocker resolved and issue still complete |
| failed | ready | issue repaired / replanned |
| any non-terminal | cancelled | Human cancels Issue |

Forbidden transitions:

```text
draft -> eligible
ready -> leased
eligible -> in-progress
leased -> done
in-progress -> done
pr -> done
checks-passing -> done
merged -> done
blocked -> in-progress
failed -> in-progress
```

Critical distinction:

```text
Ready = the issue is well-specified.
Eligible = the issue is currently allowed to execute.
Leased = the issue has been claimed by one agent.
InProgress = the agent has started an Execution Run.
Done = merge-equivalent + checks + evidence + review/update.
```

## 8. Eligibility Engine

Eligibility must be computed from current local facts.

An Issue is Eligible only if all hard gates pass:

```text
1. Project status is active.
2. Milestone status is active.
3. Issue belongs to the current active Milestone.
4. Issue status is ready.
5. All dependency Issues are done.
6. Acceptance criteria are present.
7. Test plan is present.
8. Risk level is present.
9. Rollback plan is present.
10. Expected files / affected areas are declared.
11. No unresolved blocker exists.
12. Single code-changing issue rule is satisfied.
13. If high-risk, human approval exists.
14. If blocked files are needed, human approval exists.
```

Adapter-backed gates:

```text
15. No conflicting open PR exists.
16. Repo working tree is clean.
17. Base branch is current.
18. Required check definitions exist.
19. Required credentials / environment are available.
20. Issue size is within agent execution policy.
```

Local MVP behavior:

- If an adapter-backed gate cannot be checked locally, Eligibility Snapshot must set it to `unknown`.
- Unknown adapter-backed gates may pass only when the Issue kind is non-code-changing or Human explicitly accepts the risk.
- Code-changing issues should treat unknown repo / PR / check gates as warnings in local MVP and as hard gates once provider adapters exist.

Eligibility failure reason vocabulary:

```text
project_not_active
milestone_not_active
issue_not_in_active_milestone
issue_not_ready
dependency_not_done
missing_acceptance_criteria
missing_test_plan
missing_risk_level
missing_rollback_plan
missing_expected_files
unresolved_blocker
single_code_changing_issue_violation
missing_high_risk_approval
blocked_file_requires_approval
conflicting_pr_open
repo_dirty
base_branch_stale
missing_check_definitions
missing_environment
issue_too_large
adapter_gate_unknown
```

Required output:

```yaml
issueId: ISSUE-123
eligible: false
computedAt: 2026-05-26T12:00:00Z
reasonsPassed:
  - project_active
  - milestone_active
reasonsFailed:
  - missing_test_plan
warnings:
  - adapter_gate_unknown
requiresHumanApproval: false
recommendedAction: fix-contract | wait-human | acquire-lease
```

## 9. Single Code-Changing Issue Rule

Default MVP rule:

```text
Within a Project, only one code-changing Issue may be Eligible, Leased, or InProgress at a time.
```

Allowed parallel-safe kinds:

```text
read-only-investigation
evidence-summary
milestone-review
docs-only
test-only
```

Parallel-safe requirements:

- Issue kind must be explicit.
- Expected files must not overlap with current code-changing Issue unless Human approves.
- Parallel-safe Issue must not change product behavior.
- Parallel-safe Issue must not release or renew another agent's Lease.

## 10. Lease / Lock Contract

### 10.1 Acquire Lease

An agent may acquire a Lease only when:

```text
Issue is Eligible.
No active Lease exists for the Issue.
No conflicting code-changing Lease exists for the Project.
Agent is authorized by local policy.
Lease duration is within policy.
```

Acquire output:

```yaml
leaseId: LEASE-0001
issueId: ISSUE-123
ownerAgentId: codex-local
status: active
leasedAt: 2026-05-26T12:00:00Z
expiresAt: 2026-05-26T12:45:00Z
```

### 10.2 Renew Lease

Lease renewal is allowed only if:

```text
same agent owns the lease
execution run is active
lease has not expired
progress heartbeat exists
issue has not changed scope
```

### 10.3 Release Lease

A Lease is released when:

```text
Issue reaches Done
Issue becomes Blocked
Execution Run fails
Execution Run is cancelled
Lease expires
Human cancels execution
```

### 10.4 Stale Lease Recovery

A stale Lease may be recovered when:

```text
lease expired
agent heartbeat missing
no active PR or run progress
human or system marks lease stale
```

Recovery must record:

```yaml
leaseId: LEASE-0001
status: recovered
staleRecoveryReason: "agent heartbeat missing for 30m"
recoveredBy: human | system
recoveredAt: 2026-05-26T12:45:00Z
```

### 10.5 Lease Race Conditions

The implementation must treat lease acquisition as transactional.

Minimum rule:

```text
Two agents cannot acquire an active code-changing Lease for the same Project at the same time.
```

Local file implementation must use atomic create / compare-and-write semantics or a single serialized writer.

## 11. Execution Run Contract

Before writing code, the agent must produce a run plan:

```yaml
runPlan:
  issueId: ISSUE-123
  leaseId: LEASE-0001
  goal: string
  nonGoals: string[]
  expectedFiles: string[]
  blockedFiles: string[]
  plannedSteps: string[]
  testPlan: string[]
  rollbackPlan: string
```

During execution, the agent records:

```yaml
commandsRun: []
changedFiles: []
intermediateFailures: []
testResults: []
```

The agent must stop and request human review if:

```text
scope expands beyond issue
blocked files need modification
test plan is impossible
dependency is missing
auth / billing / permission / data deletion risk appears
schema migration is needed but not declared
external network is needed but not authorized
```

Execution Run terminal outcomes:

```text
completed
failed
cancelled
blocked
```

Only `completed` may progress toward PR / Checks / Evidence.

## 12. PR / Checks Contract

An agent-created PR or local merge-equivalent record must include:

```text
Issue link
Milestone link
Summary
Acceptance criteria coverage
Changed files summary
Test plan
Test output
Risk notes
Rollback plan
Evidence checklist
```

Recommended PR template:

```md
# Summary

## Issue
Closes: <issue_id>

## Milestone
<milestone_id>

## What Changed
- ...

## Acceptance Criteria Coverage
- [ ] ...
- [ ] ...

## Tests
- [ ] lint
- [ ] typecheck
- [ ] unit tests
- [ ] integration tests
- [ ] manual verification

## Evidence
- commit:
- checks:
- logs:
- screenshots:

## Risk / Rollback
...
```

Required checks for MVP:

```text
format
lint where configured
typecheck where configured
unit tests where configured
configured project tests
security check where available and configured
```

Local MVP may satisfy checks through `agentflow verify` evidence.

## 13. Evidence Contract

An Issue is not Done until evidence is captured.

Minimum Issue evidence:

```yaml
issueId: ISSUE-123
runId: RUN-123
prUrl: null
mergeCommit: null
checksResult: passed
changedFiles:
  - path/to/file
testOutput:
  summary: "cargo test passed"
  outputPaths: []
acceptanceCriteriaCoverage:
  - criterion: "State transition guards are documented"
    status: passed
    evidence: "Workflow contract section 7"
rollbackPlan: "Revert the issue commit or restore previous docs."
knownGaps: []
```

Acceptance criteria coverage must be explicit:

```yaml
acceptanceCriteriaCoverage:
  - criterion: "User can complete login"
    status: passed
    evidence: "auth.spec.ts passed; manual login smoke completed"
  - criterion: "Invalid credentials show error"
    status: passed
    evidence: "error-state test passed"
```

Evidence rule:

```text
No evidence, no Done.
```

## 14. Milestone Review Gate

When all Issues in a Milestone are Done, AgentFlow generates a Milestone Review.

The review must include:

```text
Original milestone goal
Completed Issues
PR links or local merge-equivalent records
Merge commits where available
Checks summary
Acceptance criteria coverage
Behavior changes
Config / schema / API changes
Known risks
Deferred work
Recommendation: proceed / hold
```

Milestone Review template:

```md
# Milestone Review: <milestone>

## Original Goal
...

## Completed Issues
| Issue | PR / Merge Equivalent | Merge Commit | Status | Evidence |
|---|---|---|---|---|

## Acceptance Criteria Coverage
...

## Test Summary
...

## Behavior Changes
...

## Config / Schema / API Changes
...

## Risks
...

## Deferred Work
...

## Recommendation
Proceed to next milestone: Yes / No
```

Gate rule:

```text
Next Milestone cannot activate until Milestone Review is complete.
```

For MVP, human approval is required before proceeding to the next Milestone.

## 15. Project Closure Contract

When all Milestones are Done:

```text
Project -> Audit
```

### 15.1 Stage Code Audit

Code Audit checks:

```text
duplicate implementations
temporary code
dead code
unused exports
TODO / FIXME leftovers
test gaps
security risk
permission / auth risk
billing risk
data migration risk
performance risk
architecture drift
unexpected public API changes
```

Output:

```yaml
codeAudit:
  status: passed
  findings: []
  requiredFixes: []
  acceptedRisks: []
```

### 15.2 Root Docs Refresh

Root Docs Refresh checks and updates:

```text
README
ARCHITECTURE
CONTRIBUTING
CHANGELOG
ENV example
API docs
migration notes
runbook
known limitations
```

Output:

```yaml
docsRefresh:
  status: passed
  updatedDocs: []
  unchangedDocs: []
  missingDocs: []
  notes: []
```

### 15.3 Final Evidence Summary

Final Project summary includes:

```text
Project goal
Milestones completed
Issues completed
PRs merged or local equivalents
Checks passed
Major behavior changes
Docs updated
Known risks
Deferred work
Final recommendation
```

Project Done rule:

```text
Project cannot become Done until Code Audit, Root Docs Refresh, Final Evidence Summary, and Human Final Approval are complete.
```

### 15.4 Project Audit / Docs Refresh v0 Boundary

The v0 boundary is now fixed in `docs/specs/project-audit-docs-refresh-boundary.md`.

Minimum closure path:

```text
active
-> audit
-> docs-refresh
-> final-review
-> done
```

Boundary rules:

- Project cannot transition from `active` directly to `done`.
- Code Audit is a gate, not an automatic repair mechanism.
- Root Docs Refresh is a consistency gate, not a free-form doc rewrite pass.
- Final Evidence Summary must reference local `.agentflow/` evidence, reviews, runs, validations, and milestone summaries.
- Human Final Approval cannot be generated by the agent.
- Any code fix, docs refresh, or audit repair must go back through IssueContract and Workflow Control Core.
- v0 does not implement an automatic audit engine, Desktop execution UI, remote PR provider, GitHub issue writer, Linear writer, model call, SaaS account, payment, or cloud sync.

Next implementation slice:

```text
Project Closure State v0 实现
```

Project Closure State v0 has now implemented the local closure guard:

```text
agentflow project closure
-> .agentflow/state/project-closure.json
-> .agentflow/updates/PROJECT-CLOSURE-SUMMARY.md
```

It checks closure state only. It does not generate Code Audit, execute Root Docs Refresh, create `.agentflow/audits/`, approve final review, or mark Project done.

## 16. Human Gates

Human confirmation is required for:

```text
Project Confirmed
High-risk Issue approval
Scope expansion
Blocked-file modification
Milestone Review approval
Project Final Approval
```

Agent may propose, but Human confirms:

```text
project scope change
milestone gate change
issue acceptance criteria change
risk downgrade
skipping required evidence
marking known gaps as accepted
```

## 17. Command Boundary

Current CLI commands map to the workflow as follows:

| Command | Current role | v1 boundary |
| --- | --- | --- |
| `agentflow feature create "<goal>"` | previews or writes Product Feature -> Project / Milestones / IssueContracts | default preview only; `--write --yes` may write local facts and set active project, but must not call model or create remote objects |
| `agentflow feature status` | reads active Product Feature execution state | must not execute or write run / verify / review state |
| `agentflow feature next` | recommends next Product Feature action | must not execute; may recommend run / verify / review / wait-human |
| `agentflow plan "<intent>"` | creates IssueContract | may create draft / ready Issue only after Project and Milestone context exists |
| `agentflow goal next` | recommends next action | must not execute; may recommend plan / run / verify / review / wait-human |
| `agentflow run ISSUE-XXXX --dry-run` | creates local run artifact | future implementation must require Eligibility + Lease for code-changing runs |
| `agentflow verify ISSUE-XXXX` | records validation | contributes to Checks / Evidence |
| `agentflow review ISSUE-XXXX` | writes evidence / review / update | may mark Issue done only when evidence contract passes |
| `agentflow projects` | read-only project snapshot | must not write state |
| Desktop Workbench | read-only display | must not execute run / verify / review / merge |

Forbidden MVP command behavior:

```text
Desktop executes recommended command
goal next executes run
run bypasses IssueContract
review marks Done without evidence
plan creates remote Linear / GitHub issue
feature create calls model or bypasses IssueContract
verify calls external CI without explicit adapter and approval
```

## 18. Local File Mapping

The first implementation may remain file-backed.

Recommended local paths:

```text
.agentflow/workspace.json
.agentflow/teams/<team-id>.json
.agentflow/projects/<project-id>.json
.agentflow/milestones/<milestone-id>.json
.agentflow/issues/<issue-id>.json
.agentflow/eligibility/<issue-id>.json
.agentflow/leases/<lease-id>.json
.agentflow/runs/<run-id>/run.json
.agentflow/prs/<pr-id>.json
.agentflow/evidence/<evidence-id>.md
.agentflow/reviews/<review-id>.md
.agentflow/project-closures/<project-id>.json
.agentflow/events/events.jsonl
```

Current implementation may not have all folders yet. Missing folders are future implementation targets, not current validation failures.

## 19. Event Log Contract

The `events` stream is required for auditability.

Event shape:

```yaml
eventId: EVT-0001
workspaceId: default
projectId: agentflow-local-execution
entityType: issue
entityId: ISSUE-123
eventType: issue.eligible_computed
actorType: system
actorId: eligibility-engine
timestamp: 2026-05-26T12:00:00Z
payload:
  eligible: false
  reasonsFailed:
    - missing_test_plan
```

Recommended event names:

```text
project.created
project.confirmed
project.activated
project.entered_audit
project.entered_docs_refresh
project.final_review_started
project.done
project.blocked
project.failed

milestone.created
milestone.ready
milestone.activated
milestone.review_started
milestone.review_completed
milestone.done
milestone.blocked
milestone.failed

issue.created
issue.ready
issue.eligible_computed
issue.leased
issue.execution_started
issue.pr_opened
issue.checks_passed
issue.merged
issue.evidence_captured
issue.done
issue.blocked
issue.failed
issue.needs_human_review

lease.acquired
lease.renewed
lease.released
lease.expired
lease.recovered

run.started
run.completed
run.failed
run.blocked

evidence.created

audit.started
audit.completed
docs_refresh.started
docs_refresh.completed
```

Event rules:

- State transitions must emit events.
- Human gates must emit events.
- Lease recovery must emit events.
- Evidence creation must emit events.
- Events are append-only.

## 20. Minimal Data Model

A first implementation can start with these tables, collections, or file-backed equivalents:

```text
workspaces
teams
projects
milestones
issues
issue_dependencies
eligibility_snapshots
leases
execution_runs
pull_requests
check_results
evidence
milestone_reviews
project_closures
events
```

Minimal indexes:

```text
issues(project_id, milestone_id, status, kind)
leases(project_id, issue_id, status)
execution_runs(issue_id, status)
evidence(issue_id, run_id)
events(entity_type, entity_id, timestamp)
milestones(project_id, status)
```

## 21. MVP Implementation Order

### Phase 1: Workflow State Machine

Build:

```text
Project state machine
Milestone state machine
Issue state machine
transition guard functions
event log append for transitions
state machine tests
```

Do not build UI first.

### Phase 2: Eligibility Engine

Build:

```text
eligibility computation
eligibility failure reasons
active milestone check
dependency check
repo status check
conflicting PR check
issue completeness check
single code-changing issue rule
```

### Phase 3: Lease / Lock

Build:

```text
lease acquire
lease renew
lease release
lease expiry
stale lease recovery
agent heartbeat
transaction / atomic write boundary
```

### Phase 4: Execution Evidence

Build:

```text
execution run record
PR / local merge-equivalent capture
checks capture
commit hash capture
changed files capture
test output capture
acceptance coverage capture
rollback plan capture
```

### Phase 5: Milestone / Project Closure

Build:

```text
milestone review generation
milestone review gate
project code audit stage
root docs refresh stage
final evidence summary
human final approval
```

## 22. First Executable Vertical Slice

The first AIE implementation should prove this sequence:

```text
Create Project
Confirm Project
Create Milestone
Mark Milestone Ready
Activate Milestone
Create Issue
Mark Issue Ready
Compute Eligible
Acquire Lease
Start Execution Run
Record checks / validation
Attach PR or local merge-equivalent
Capture Evidence
Mark Issue Done
Generate Milestone Review
Mark Milestone Done
Enter Project Audit
Enter Docs Refresh
Generate Final Evidence Summary
Mark Project Done
```

Vertical slice acceptance:

```text
Every state transition is guarded.
Every state transition emits an event.
Eligibility is recomputable.
Lease acquisition is exclusive for code-changing issues.
Issue Done fails without Evidence.
Milestone Done fails without Review.
Project Done fails without Audit / Docs Refresh / Final Evidence.
```

## 23. PRD Acceptance Criteria for v1

This Workflow Contract v1 is complete when it defines:

```text
Project / Milestone / Issue lifecycle
Ready vs Eligible vs Leased distinction
Eligibility computation rules
Lease ownership and stale recovery
Execution Run requirements
PR / checks requirements
Evidence requirements
Milestone Review gate
Project Audit / Docs Refresh closure
Human approval gates
MVP non-goals
System invariants
Local file mapping
Event log contract
Minimal data model
First executable vertical slice
```

## 24. ARC Review Checklist

`@005 / ARC` should review this contract for:

```text
state machine consistency
invalid transition prevention
eligibility rule completeness
lease race conditions
idempotency
event sourcing / auditability
repo integration boundaries
GitHub / PR / check abstraction
evidence immutability
human override policy
failure recovery
security-sensitive issue handling
file-backed atomicity
future database migration path
Desktop read-only boundary
```

Architecture questions to answer:

```text
Is Eligible stored or computed on demand?
Are state transitions command-based or directly mutable?
Are leases transactional?
How are stale leases recovered safely?
How do we detect conflicting PRs?
How do we classify code-changing vs non-code-changing issues?
How is evidence made tamper-resistant enough for local MVP?
What is the minimum Git provider abstraction?
What does an agent need to read / write?
What must remain human-only?
Which state transitions are synchronous vs derived?
Which events are required for replay?
```

## 25. AIE Implementation Brief

`@000 / AIE` should not start with UI.

Implementation should start in `agentflow-core`.

Initial modules:

```text
state_machine
eligibility_engine
lease_manager
execution_run_recorder
evidence_capture
milestone_review_generator
project_closure_orchestrator
event_log
```

Initial CLI surface:

```text
agentflow state check
agentflow eligibility
agentflow eligibility ISSUE-XXXX
agentflow lease
agentflow events tail
```

Workflow Control Core v0 has implemented the local state / eligibility / lease / run / evidence chain. `agentflow lease acquire` and `agentflow lease release` remain internalized in `run` / `review` for v0; explicit mutation subcommands are still future work.

## 26. Verification Matrix

Workflow State Machine validation:

```text
project draft cannot jump to active
project active cannot jump to done
milestone ready cannot jump to done
issue ready cannot jump to leased without eligibility
issue merged cannot jump to done without evidence
terminal cancelled / failed cannot reactivate without explicit repair contract
```

Eligibility Engine validation:

```text
missing test plan fails eligibility
dependency not done fails eligibility
issue outside active milestone fails eligibility
single code-changing issue violation fails eligibility
high-risk issue without human approval fails eligibility
non-code-changing issue can be parallel-safe when file overlap is absent
```

Lease validation:

```text
eligible issue can acquire lease
non-eligible issue cannot acquire lease
second code-changing lease in same project fails
lease renewal by non-owner fails
expired lease can be recovered with reason
release writes event
```

Workflow Control Core v0 validation:

```text
agentflow eligibility computes eligible / not eligible candidates
agentflow lease reports active and stale local leases
agentflow run requires eligibility before creating RUN-XXXX
agentflow run acquires .agentflow/leases/LEASE-*.json
AgentRun records projectId / milestoneId / leaseId
agentflow review releases the lease after evidence and review are captured
goal next uses eligibility for active milestone queue recommendations
```

Evidence validation:

```text
issue done without evidence fails
evidence without acceptance coverage fails
evidence with failed required checks fails done gate
milestone review cannot generate until all issues done
project closure cannot complete without audit / docs refresh / final approval
```

## 27. Next Authorized Planning Step

The next implementation planning step is:

```bash
agentflow plan "Workflow State Machine v0 边界定义"
```

That next step must only define the Project / Milestone / Issue state machine boundary and transition guards. It must not implement UI, lease writer, provider adapters, or automatic execution.
