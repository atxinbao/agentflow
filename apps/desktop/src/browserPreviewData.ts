import type {
  PanelContextPack,
  PanelManifestSnapshot,
  PanelSearchSnapshot,
  PanelStatusSnapshot,
  ProjectDirectoryPage,
  ProjectFileChild,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileSearchSnapshot,
  ProjectFileTextRange,
  ProjectFileViewMode,
  ProjectFilesSnapshot,
  AgentEnvironmentStatus,
  InputIssue,
  InputProject,
  InputSnapshot,
  InputStatusSnapshot,
  McpSessionSnapshot,
  IssueStatusIndex,
  ExecuteStatusSnapshot,
  OutputIndex,
  OutputIndexEntry,
  AuditIndex,
  HumanAuditReport,
  ProjectProjection,
  StateStatusSnapshot,
  TaskProjection,
  TaskTimelineItem,
} from "./types";
import {
  getProjectFileExtensionFromName,
  normalizeProjectRelativePath,
} from "./features/project-files/model/projectFileUtils";

export const BROWSER_PREVIEW_PROJECT_ROOT = "/Users/mac/Documents/AgentFlow";

const previewTimestamp = 1780291200;
const previewDoneIssueId = "iss-done";
const previewAuditId = "audit-browser-preview-001";
const previewProgressRunId = "run-browser-preview-001";
const previewReviewRunId = "run-browser-preview-002";
const previewDoneRunId = "run-browser-preview-003";
const previewDeliveryRunId = previewDoneRunId;
const previewProjectId = "project-browser-preview";
const previewSpecId = "spec-browser-preview";
const previewProjectIssueIds = ["iss-ready", "iss-progress", "iss-review", "iss-audit-ready", "iss-done", "iss-cancel"];
const previewIssueScope = [
  "展示任务状态流和项目分组。",
  "展示任务页里的当前阶段、最终交付和高级详情。",
  "保持真实桌面客户端只读取真实本地数据。",
];
const previewIssueNonGoals = ["不执行命令。", "不写入本地工作区。", "不调用模型。", "不创建远程对象。"];
const previewIssueAcceptanceCriteria = [
  "浏览器预览可展示任务工作台新结构。",
  "任务页主链路展示状态流、当前阶段和最终交付。",
  "导航收口后任务、文件和审计入口仍可用，执行与交付在任务页内展示。",
];
const previewIssueValidationCommands = [
  "npm --prefix apps/desktop run build",
  "git diff --check",
];

export type BrowserPreviewTaskHierarchyScenario =
  | "default"
  | "empty"
  | "ungrouped"
  | "empty-project"
  | "missing-issue";

const browserPreviewTaskHierarchyScenarios = new Set<BrowserPreviewTaskHierarchyScenario>([
  "default",
  "empty",
  "ungrouped",
  "empty-project",
  "missing-issue",
]);

export function resolveBrowserPreviewTaskHierarchyScenario(search?: string | null): BrowserPreviewTaskHierarchyScenario {
  const query = search ?? "";
  const params = new URLSearchParams(query.startsWith("?") ? query : `?${query}`);
  const value = params.get("taskHierarchyScenario") ?? params.get("taskScenario") ?? "default";
  return browserPreviewTaskHierarchyScenarios.has(value as BrowserPreviewTaskHierarchyScenario)
    ? (value as BrowserPreviewTaskHierarchyScenario)
    : "default";
}

export function currentBrowserPreviewTaskHierarchyScenario(): BrowserPreviewTaskHierarchyScenario {
  if (typeof window === "undefined") {
    return "default";
  }
  return resolveBrowserPreviewTaskHierarchyScenario(window.location.search);
}

const previewInputIssues: InputIssue[] = [
  browserPreviewInputIssue("iss-backlog", "整理需求入口", "backlog", "backlog", "需求已创建，等待整理成 SPEC。", {
    priority: "p3",
  }),
  browserPreviewInputIssue("iss-ready", "生成执行任务包", "todo", "todo", "SPEC 已确认，可以交给 Agent。", {
    blocks: ["iss-progress"],
    issueModel: "project",
    priority: "p1",
    projectId: previewProjectId,
  }),
  browserPreviewInputIssue("iss-progress", "执行受控改动", "todo", "in_progress", "执行助手已接手任务。", {
    blockedBy: ["iss-ready"],
    blocks: ["iss-review"],
    issueModel: "project",
    priority: "p0",
    projectId: previewProjectId,
    executionRisk: "medium",
  }),
  browserPreviewInputIssue("iss-review", "整理评审交付", "todo", "in_review", "执行和本地验证已完成，等待 PR/MR 合并。", {
    blockedBy: ["iss-progress"],
    issueModel: "project",
    projectId: previewProjectId,
  }),
  browserPreviewInputIssue("iss-audit-ready", "复制审计任务包", "todo", "todo", "审计任务可以交给 Audit Agent。", {
    audit: {
      auditId: previewAuditId,
      auditOutputDir: `.agentflow/audit/${previewAuditId}`,
      expectedOutputs: previewAuditExpectedOutputs(previewAuditId),
      sourceDeliveryPath: "CHANGELOG.md",
      sourceReleaseId: previewDeliveryRunId,
      trigger: "human-via-agent",
    },
    issueCategory: "audit",
    issueModel: "project",
    projectId: previewProjectId,
    requiredAgentRole: "audit-agent",
  }),
  browserPreviewInputIssue("iss-done", "确认交付完成", "done", "done", "交付已写回，后续审计保持独立入口。", {
    issueModel: "project",
    projectId: previewProjectId,
  }),
  browserPreviewInputIssue("iss-cancel", "取消过期需求", "cancel", "cancel", "任务已取消。", {
    issueModel: "project",
    projectId: previewProjectId,
  }),
];

function browserPreviewInputIssue(
  issueId: string,
  title: string,
  status: InputIssue["status"],
  displayStatus: InputIssue["displayStatus"],
  summary: string,
  options: {
    audit?: InputIssue["audit"];
    blockedBy?: string[];
    blocks?: string[];
    expectedOutputs?: InputIssue["expectedOutputs"];
    issueCategory?: InputIssue["issueCategory"];
    issueModel?: InputIssue["issueModel"];
    projectId?: string | null;
    priority?: string;
    requiredAgentRole?: InputIssue["requiredAgentRole"];
    executionRisk?: string;
  } = {},
): InputIssue {
  const issueCategory = options.issueCategory ?? "spec";
  return {
    version: "input-issue.browser-preview",
    issueId,
    issueModel: options.issueModel ?? "direct",
    issueCategory,
    requiredAgentRole: options.requiredAgentRole ?? (issueCategory === "audit" ? "audit-agent" : "build-agent"),
    sourceSpecId: previewSpecId,
    projectId: options.projectId ?? null,
    title,
    summary,
    kind: "feature",
    priority: options.priority ?? "p2",
    status,
    displayStatus,
    executionRisk: options.executionRisk ?? "low",
    expectedOutputs: options.expectedOutputs ?? (issueCategory === "spec" ? previewBuildExpectedOutputs(issueId) : undefined),
    scope: previewIssueScope,
    nonGoals: previewIssueNonGoals,
    acceptanceCriteria: previewIssueAcceptanceCriteria,
    validationHints: previewIssueValidationCommands,
    validationCommands: previewIssueValidationCommands,
    relations: {
      blockedBy: options.blockedBy ?? [],
      blocks: options.blocks ?? [],
      related: [],
      duplicateOf: null,
    },
    panel: {
      snapshotId: null,
      contextPackId: null,
    },
    audit: options.audit ?? null,
    system: {
      createdBy: "browser-preview",
      createdAt: previewTimestamp,
      updatedAt: previewTimestamp,
      path: `.agentflow/spec/issues/${issueId}.json`,
      revision: 1,
    },
  };
}

function previewBuildExpectedOutputs(issueId: string) {
  return {
    evidencePath: `.agentflow/tasks/${issueId}/evidence/evidence.json`,
    executeRunDir: `.agentflow/tasks/${issueId}/runs/${issueId}`,
    publicDeliveryRecord: "PR/MR body or CHANGELOG.md",
  };
}

function previewAuditExpectedOutputs(auditId: string) {
  const outputDir = `.agentflow/audit/${auditId}`;
  return {
    "audit-report.md": `${outputDir}/audit-report.md`,
    "audit.json": `${outputDir}/audit.json`,
    "evidence-map.json": `${outputDir}/evidence-map.json`,
    "findings.json": `${outputDir}/findings.json`,
    "traceability.json": `${outputDir}/traceability.json`,
  };
}

function browserPreviewInputProject(
  options: {
    issueIds?: string[];
    objective?: string;
    projectId?: string;
    summary?: string;
    title?: string;
  } = {},
): InputProject {
  const projectId = options.projectId ?? previewProjectId;
  return {
    version: "input-project.browser-preview",
    projectId,
    sourceSpecId: previewSpecId,
    title: options.title ?? "浏览器预览任务项目",
    summary: options.summary ?? "用于验证任务页里的项目分组和任务工作流。",
    objective: options.objective ?? "在任务页展示项目分组、项目调度视图和任务工作流。",
    scope: ["展示项目调度概览。", "展示任务页内的当前阶段与最终交付主链路。", "验证查看当前任务按钮会切换到任务工作流。"],
    nonGoals: ["不写入真实 .agentflow/spec。", "不创建远程对象。"],
    successCriteria: ["项目行可选中。", "右侧可显示项目调度视图。", "查看当前任务后右侧显示任务工作流。"],
    issueIds: options.issueIds ?? previewProjectIssueIds,
    status: "active",
    panel: {
      snapshotId: null,
      contextPackId: null,
    },
    system: {
      createdBy: "browser-preview",
      createdAt: previewTimestamp,
      updatedAt: previewTimestamp,
      path: `.agentflow/spec/projects/${projectId}.json`,
      revision: 1,
    },
  };
}

function browserPreviewIssuesForScenario(scenario: BrowserPreviewTaskHierarchyScenario) {
  if (scenario === "empty" || scenario === "empty-project" || scenario === "missing-issue") {
    return [];
  }
  if (scenario === "ungrouped") {
    return [previewInputIssues[0]];
  }
  return previewInputIssues;
}

function browserPreviewProjectsForScenario(scenario: BrowserPreviewTaskHierarchyScenario) {
  if (scenario === "empty" || scenario === "ungrouped") {
    return [];
  }
  if (scenario === "empty-project") {
    return [
      browserPreviewInputProject({
        issueIds: [],
        objective: "验证 Project 存在但还没有任何 Issue 时的空态。",
        projectId: "project-empty-browser-preview",
        summary: "用于验证 Project 下没有 Issue 的空态。",
        title: "空任务项目",
      }),
    ];
  }
  if (scenario === "missing-issue") {
    return [
      browserPreviewInputProject({
        issueIds: ["iss-missing-browser-preview"],
        objective: "验证 Project 引用缺失 Issue 时的 warning。",
        projectId: "project-missing-browser-preview",
        summary: "用于验证 Project 引用缺失 Issue 的 warning。",
        title: "缺失引用项目",
      }),
    ];
  }
  return [browserPreviewInputProject()];
}

function browserPreviewRelationsForScenario(scenario: BrowserPreviewTaskHierarchyScenario) {
  if (scenario !== "default") {
    return [];
  }
  return [
    { fromIssueId: "iss-progress", toIssueId: "iss-ready", type: "blocked-by" as const },
    { fromIssueId: "iss-progress", toIssueId: "iss-review", type: "blocks" as const },
  ];
}

export function createBrowserPreviewProjectFilesSnapshot(
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  viewMode: ProjectFileViewMode | string = "source",
): ProjectFilesSnapshot {
  return {
    version: "project-files.browser-preview",
    projectRoot,
    selectedPath: "README.md",
    entries: filterBrowserPreviewEntries(browserPreviewTopLevelEntries(), viewMode),
  };
}

export function createBrowserPreviewPanelStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): PanelStatusSnapshot {
  return {
    version: "panel-status.browser-preview",
    projectRoot,
    status: "ready",
    fileCount: 9,
    symbolCount: 18,
    relationCount: 12,
    updatedAt: previewTimestamp,
    lastError: null,
    watcherStatus: "mock",
    watcherBackend: "browser-preview",
    preflightStatus: "ready",
    protectionStatus: "ready",
    warnings: [],
  };
}

export function createBrowserPreviewAgentEnvironmentStatus(
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
): AgentEnvironmentStatus {
  const agentLocale = globalThis.navigator?.language || "zh-CN";
  return {
    version: "agent-environment-status.browser-preview",
    projectRoot,
    status: "ready",
    ready: true,
    checkedAt: previewTimestamp,
    repairedAt: null,
    agentMd: {
      exists: true,
      managed: true,
      version: "agent-entry.v2",
      hash: "browser-preview-agents-md",
      backedUp: false,
      trackedByGit: false,
    },
    manual: {
      exists: true,
      path: ".agentflow/define/agent/Agentflow.md",
      hash: "browser-preview-agentflow-manual",
    },
    skillsLock: {
      exists: true,
      valid: true,
      path: ".agentflow/define/agent/skills-lock.json",
      skillCount: 7,
    },
    skills: [
      "request-triage",
      "requirement-intake-filter",
      "spec-gate-authoring",
      "input-issue-generation",
      "boundary-check",
      "validation",
      "plain-work-style",
    ].map((name) => ({
      name,
      path: `.agentflow/define/agent/skills/${name}/SKILL.md`,
      exists: true,
      hashMatches: true,
      version: "v1",
    })),
    repairs: [],
    warnings: ["浏览器预览只展示模拟的 Agent Manual 状态，不会写入 AGENTS.md。"],
    errors: [],
    workspaceManifest: {
      exists: true,
      path: ".agentflow/workspace-manifest.json",
      valid: true,
      layoutVersion: "agentflow-layout.v1",
    },
    ownership: {
      version: "agentflow-workspace-ownership.browser-preview",
      projectRoot,
      status: "managed-current",
      readyForPrepare: true,
      agentBlocked: false,
      agentflowPath: `${projectRoot}/.agentflow`,
      marker: {
        manifestExists: true,
        manifestManagedByAgentflow: true,
        manifestVersion: "agentflow-workspace-manifest.v1",
        layoutVersion: "agentflow-layout.v1",
        agentManualExists: true,
        skillsLockExists: true,
        managedEntryExists: true,
      },
      detectedFiles: [
        ".agentflow/workspace-manifest.json",
        ".agentflow/define/agent/Agentflow.md",
        ".agentflow/define/agent/skills-lock.json",
      ],
      warnings: [],
      errors: [],
      recommendedAction: "validate-repair",
    },
    layout: {
      version: "agentflow-layout.v1",
      ready: true,
      createdPaths: [],
      reusedPaths: [
        ".agentflow/define/spec/SPEC.md",
        ".agentflow/define/tdd/TDD.md",
        ".agentflow/define/release/RELEASE.md",
        ".agentflow/define/audit/AUDIT.md",
        ".agentflow/spec",
        ".agentflow/spec/manifest.json",
        ".agentflow/spec/index.json",
        ".agentflow/panel",
        ".agentflow/tasks",
        ".agentflow/state",
      ],
      missingPaths: [],
    },
    locale: {
      version: "agent-locale.v1",
      agentLocale,
      rawOsLocale: agentLocale,
      manualLanguage: "en",
      source: "browser-preview",
      checkedAt: previewTimestamp,
      fallback: false,
      warnings: [],
    },
    style: {
      version: "agent-style.v1",
      styleId: "plain-work-style",
      manualLanguage: "en",
      appliesToAgentLocale: true,
      appliesToCodeComments: true,
      checkedAt: previewTimestamp,
      warnings: [],
    },
    shadowGuard: {
      checked: [
        ".rules",
        ".cursorrules",
        ".windsurfrules",
        ".clinerules",
        ".github/copilot-instructions.md",
        "AGENT.md",
        "CLAUDE.md",
        "GEMINI.md",
      ],
      detected: [],
    },
  };
}

export function createBrowserPreviewInputStatus(
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  scenario = currentBrowserPreviewTaskHierarchyScenario(),
): InputStatusSnapshot {
  const issues = browserPreviewIssuesForScenario(scenario);
  const projects = browserPreviewProjectsForScenario(scenario);
  return {
    version: "input-status.browser-preview",
    projectRoot,
    status: "ready",
    ready: true,
    manifestExists: true,
    indexExists: true,
    summary: {
      intake: 1,
      draftSpecs: 1,
      approvedSpecs: 1,
      projects: projects.length,
      issues: issues.length,
      blockedIssues: 0,
      highRiskIssues: 0,
    },
    missingPaths: [],
    warnings: ["浏览器预览只展示 mock spec 状态，不写 .agentflow/spec。"],
    errors: [],
  };
}

export function createBrowserPreviewInputSnapshot(
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  scenario = currentBrowserPreviewTaskHierarchyScenario(),
): InputSnapshot {
  const issues = browserPreviewIssuesForScenario(scenario);
  const projects = browserPreviewProjectsForScenario(scenario);
  const relations = browserPreviewRelationsForScenario(scenario);
  return {
    version: "input-snapshot.browser-preview",
    projectRoot,
    ready: true,
    status: createBrowserPreviewInputStatus(projectRoot, scenario),
    manifest: {
      version: "input-manifest.browser-preview",
      projectRoot,
      status: "ready",
    },
    index: {
      version: "input-index.browser-preview",
      issues: issues.map((issue) => ({
        id: issue.issueId,
        title: issue.title,
        path: issue.system?.path ?? `.agentflow/spec/issues/${issue.issueId}.json`,
        status: issue.status,
        displayStatus: issue.displayStatus,
      })),
    },
    intake: [],
    specs: [],
    projects,
    issues,
    relations: {
      version: "input-issue-relations.browser-preview",
      relations,
    },
  };
}

export function createBrowserPreviewIssueStatusIndex(
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  scenario = currentBrowserPreviewTaskHierarchyScenario(),
): IssueStatusIndex {
  const issues = browserPreviewIssuesForScenario(scenario);
  return {
    version: "state-issue-status-index.browser-preview",
    updatedAt: previewTimestamp,
    issues: issues.map((issue) => ({
      issueId: issue.issueId,
      displayStatus: issue.displayStatus,
      priority: issue.priority,
      executionRisk: issue.executionRisk,
      latestRunId: previewRunIdForIssue(issue),
      executeStatus:
        issue.displayStatus === "in_progress"
          ? "running"
          : issue.displayStatus === "in_review" || issue.displayStatus === "done"
            ? "completed"
            : null,
      evidenceStatus: issue.displayStatus === "in_review" || issue.displayStatus === "done" ? "complete" : "missing",
      deliveryStatus:
        issue.displayStatus === "done"
          ? "published"
          : issue.displayStatus === "in_review"
            ? "ready"
            : "missing",
      auditStatus: issue.displayStatus === "done" ? "passed" : "not-requested",
    })),
  };
}

export function createBrowserPreviewTaskProjection(
  issueId: string,
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  scenario = currentBrowserPreviewTaskHierarchyScenario(),
): TaskProjection | null {
  void projectRoot;
  const issue = browserPreviewIssuesForScenario(scenario).find((item) => item.issueId === issueId);
  if (!issue) {
    return null;
  }
  const currentState = issue.displayStatus;
  const runId = previewRunIdForIssue(issue);
  return {
    version: "task-projection.browser-preview",
    issueId,
    projectId: issue.projectId ?? null,
    workflowRef: "build-agent.issue-loop.v1",
    currentState,
    displayStatus: currentState,
    currentTransition: browserPreviewTransitionForState(currentState),
    latestRunId: runId,
    branchName: runId ? `agentflow/browser-preview/${issueId}` : null,
    timeline: browserPreviewTimelineForIssue(issue),
    publicDelivery: {
      evidencePath: runId ? `.agentflow/tasks/${issueId}/evidence/evidence.json` : null,
      prUrl: currentState === "in_review" || currentState === "done" ? "https://github.com/example/agentflow/pull/100" : null,
      mergeCommit: currentState === "done" ? "426b217f" : null,
      changelogPath: currentState === "in_review" || currentState === "done" ? "CHANGELOG.md" : null,
      releaseNotesUrl: null,
    },
    runtime: {
      runId,
      runStatus:
        currentState === "done"
          ? "completed"
          : currentState === "in_review"
            ? "completed"
            : currentState === "in_progress"
              ? "in_progress"
              : currentState === "todo"
                ? "queued"
                : "missing",
      branchName: runId ? `agentflow/browser-preview/${issueId}` : null,
      checkpointCount: currentState === "in_progress" ? 2 : currentState === "in_review" || currentState === "done" ? 3 : 0,
      latestCheckpointId: currentState === "in_progress" || currentState === "in_review" || currentState === "done" ? "checkpoint-003" : null,
      latestCheckpointState:
        currentState === "done" ? "done" : currentState === "in_review" ? "in_review" : currentState === "in_progress" ? "in_progress" : null,
      latestCheckpointSummary:
        currentState === "done"
          ? "验证通过并完成写回。"
          : currentState === "in_review"
            ? "本地验证通过，等待 PR/MR 合并。"
            : currentState === "in_progress"
              ? "当前处在执行中。"
              : null,
    },
    session: {
      provider: runId ? "codex" : null,
      sessionId: runId ? `codex-${runId}` : null,
      status:
        currentState === "done"
          ? "done"
          : currentState === "in_review"
            ? "in-review"
            : currentState === "in_progress"
              ? "running"
              : null,
      launchRequestedAt: runId ? previewTimestamp + 120 : null,
      claimedAt: runId ? previewTimestamp + 150 : null,
      createdAt: runId ? previewTimestamp + 180 : null,
      updatedAt: runId ? previewTimestamp + 360 : null,
      launchRequestPath: runId ? `.agentflow/tasks/${issueId}/runs/${runId}/launch/agent-request.json` : null,
      planPath: runId ? `.agentflow/tasks/${issueId}/runs/${runId}/plan.md` : null,
      logPath: runId ? `.agentflow/tasks/${issueId}/evidence/verify.log` : null,
      branchName: runId ? `agentflow/browser-preview/${issueId}` : null,
    },
    delivery: {
      status: currentState === "done" ? "published" : currentState === "in_review" ? "ready" : "missing",
      evidenceStatus: runId ? "ready" : "missing",
      evidencePath: runId ? `.agentflow/tasks/${issueId}/evidence/evidence.json` : null,
      prUrl: currentState === "in_review" || currentState === "done" ? "https://github.com/example/agentflow/pull/100" : null,
      mergeCommit: currentState === "done" ? "426b217f" : null,
      publicRecordPath: currentState === "in_review" || currentState === "done" ? "CHANGELOG.md" : null,
      summaryLine:
        currentState === "done"
          ? "公开交付已整理到 PR/MR body、CHANGELOG.md。"
          : currentState === "in_review"
            ? "公开交付正在整理，等待写入 CHANGELOG.md。"
            : "当前还没有公开交付记录。",
      publicRecordItems:
        currentState === "in_review" || currentState === "done"
          ? ["PR/MR body", "CHANGELOG.md"]
          : [],
      missingPublicRecords:
        currentState === "in_review"
          ? ["CHANGELOG.md 或 release notes"]
          : [],
      currentIssueId: null,
      publishedCount: 0,
      readyCount: 0,
      missingCount: 0,
    },
    audit: {
      status: currentState === "done" ? "not-requested" : "not-requested",
      latestAuditId: null,
      sourceIssueId: null,
      reportPath: null,
      requestedAt: null,
      summaryLine: "当前没有审计请求。",
      findingsCount: 0,
      findings: [],
      evidenceGaps: [],
      repairRecommendations: [],
    },
    updatedAt: previewTimestamp + 360,
  };
}

export function createBrowserPreviewProjectProjection(
  projectId: string,
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  scenario = currentBrowserPreviewTaskHierarchyScenario(),
): ProjectProjection | null {
  void projectRoot;
  if (projectId !== previewProjectId) {
    return null;
  }
  const issues = browserPreviewIssuesForScenario(scenario).filter((issue) => issue.projectId === projectId);
  const current = issues.filter((issue) => ["todo", "in_progress", "in_review", "blocked"].includes(issue.displayStatus)).map((issue) => issue.issueId);
  const past = issues.filter((issue) => ["done", "cancel"].includes(issue.displayStatus)).map((issue) => issue.issueId);
  const future = issues.filter((issue) => issue.displayStatus === "backlog").map((issue) => issue.issueId);
  const blocked = issues.filter((issue) => issue.displayStatus === "blocked").map((issue) => issue.issueId);
  const activeIssueId = browserPreviewActiveProjectIssueId(issues);
  const completedIssueCount = past.filter((issueId) => {
    const issue = issues.find((entry) => entry.issueId === issueId);
    return issue?.displayStatus === "done";
  }).length;
  const canceledIssueCount = past.filter((issueId) => {
    const issue = issues.find((entry) => entry.issueId === issueId);
    return issue?.displayStatus === "cancel";
  }).length;
  const remainingIssueCount = Math.max(issues.length - completedIssueCount - canceledIssueCount, 0);
  const completion =
    completedIssueCount === issues.length && issues.length
      ? {
          currentState: "goal-recheck",
          latestOutcome: null,
          nextRecommendedAction: "enter-completion-decision",
          nextRecommendedActionLabel: "进入完成判断",
          nextRecommendedActionReason: "当前任务已经收口，下一步由 Goal Agent 明确给出项目完成决策。",
          totalIssueCount: issues.length,
          completedIssueCount,
          canceledIssueCount,
          remainingIssueCount,
          blockedIssueCount: blocked.length,
          openQuestions: [
            "当前交付是否真正满足 GOAL.md 和 PLAN.md？",
            "项目应该接受、继续、调整、暂停，还是进入下一阶段？",
          ],
          rationale: [
            "任务执行已经收口，但交付是否满足 Goal / Plan 还需要重新判断。",
            "Project 完成必须由 Goal Agent 显式给出 completion decision。",
          ],
          updatedAt: previewTimestamp + 420,
        }
      : {
          currentState: "continue",
          latestOutcome: "continue",
          nextRecommendedAction: "start-project-loop",
          nextRecommendedActionLabel: "继续项目循环",
          nextRecommendedActionReason: `当前还有 ${remainingIssueCount} 条任务未完成，先继续推进任务循环。`,
          totalIssueCount: issues.length,
          completedIssueCount,
          canceledIssueCount,
          remainingIssueCount,
          blockedIssueCount: blocked.length,
          openQuestions: [],
          rationale: [`当前还有 ${remainingIssueCount} 条任务未完成，Completion Decision 暂时不能收口项目。`],
          updatedAt: previewTimestamp + 420,
        };
  const deliverySummaryLine =
    completedIssueCount === issues.length && issues.length
      ? "项目公开交付已汇总到 PR/MR body、CHANGELOG.md。"
      : current.length
        ? `项目公开交付仍在围绕 ${activeIssueId ?? current[0]} 整理。`
        : "当前项目还没有公开交付记录。";
  return {
    version: "project-projection.v3.browser-preview",
    projectId,
    title: "任务中心工作台优化重构",
    objective: "让任务页按状态流统一展示执行、交付和审计摘要。",
    status: current.length ? "active" : future.length ? "planned" : "done",
    stageKey: current.length ? "active" : future.length ? "ready-to-start" : "completion-ready",
    stageLabel: current.length ? "正在推进" : future.length ? "准备开工" : "等待完成判断",
    stageSummary: activeIssueId
      ? `当前项目围绕 ${activeIssueId} 推进。`
      : future.length
        ? `当前还没有活跃任务，下一条待启动任务是 ${future[0]}。`
        : "全部任务已完成，下一步进入完成判断。",
    issueIds: issues.map((issue) => issue.issueId),
    currentIssueId: activeIssueId,
    lanes: {
      current,
      past,
      future,
      blocked,
    },
    nextAction: activeIssueId ? `继续推进 ${activeIssueId}。` : future.length ? `启动 ${future[0]}。` : "进入完成判断",
    nextActionLabel: activeIssueId ? "继续当前任务" : future.length ? "启动下一条任务" : "进入完成判断",
    nextActionReason: activeIssueId
      ? `当前活跃任务是 ${activeIssueId}，项目下一步仍然围绕它推进。`
      : future.length
        ? `${future[0]} 当前是项目下一条最直接的推进入口。`
        : "任务已全部完成，等待完成判断收口。",
    blockers: blocked.map((issueId) => ({ issueId, reason: "等待阻断条件解除。" })),
    completionHint: `${completion.nextRecommendedActionReason} 最近交付：${deliverySummaryLine}`,
    completion,
    delivery: {
      status: completedIssueCount === issues.length && issues.length ? "published" : current.length ? "ready" : "missing",
      evidenceStatus: "ready",
      evidencePath: null,
      prUrl: null,
      mergeCommit: null,
      publicRecordPath: completedIssueCount ? "CHANGELOG.md" : null,
      summaryLine: deliverySummaryLine,
      publicRecordItems: completedIssueCount ? ["PR/MR body", "CHANGELOG.md"] : [],
      missingPublicRecords: current.length ? ["CHANGELOG.md 或 release notes"] : [],
      currentIssueId: activeIssueId,
      publishedCount: completedIssueCount,
      readyCount: current.length,
      missingCount: future.length,
    },
    audit: {
      status: "not-requested",
      latestAuditId: null,
      sourceIssueId: null,
      reportPath: null,
      requestedAt: null,
      summaryLine: "当前没有审计请求。",
      findingsCount: 0,
      findings: [],
      evidenceGaps: [],
      repairRecommendations: [],
    },
    issueCount: issues.length,
    completedIssueCount,
    projectBrain: {
      projectPath: "docs/projects/project-browser-preview",
      goalPath: "docs/projects/project-browser-preview/GOAL.md",
      planPath: "docs/projects/project-browser-preview/PLAN.md",
      decisionsPath: "docs/projects/project-browser-preview/DECISIONS.md",
      healthPath: "docs/projects/project-browser-preview/PROJECT_HEALTH.md",
      brainStatus: "ready-for-project-loop",
      goalStatus: "confirmed",
      planStatus: "confirmed",
      decisionStatus: "confirmed",
      healthStatus: "confirmed",
      missingDocuments: [],
      openQuestions: [],
      nextRecommendedAction: "start-project-loop",
      nextRecommendedActionLabel: "进入项目循环",
      nextRecommendedActionReason: "Goal / Plan / Decisions 已就绪，可以继续推进项目循环。",
      readonly: true,
    },
    updatedAt: previewTimestamp + 420,
  };
}

function browserPreviewActiveProjectIssueId(issues: InputIssue[]) {
  const priority: Record<string, number> = {
    in_progress: 0,
    in_review: 1,
    todo: 2,
    blocked: 3,
  };
  return issues
    .filter((issue) => Object.hasOwn(priority, issue.displayStatus))
    .sort((left, right) => {
      const leftRank = priority[left.displayStatus] ?? Number.MAX_SAFE_INTEGER;
      const rightRank = priority[right.displayStatus] ?? Number.MAX_SAFE_INTEGER;
      return leftRank - rightRank;
    })[0]?.issueId ?? null;
}

function browserPreviewTimelineForIssue(issue: InputIssue): TaskTimelineItem[] {
  const states: TaskTimelineItem["state"][] = ["backlog", "todo", "in_progress", "in_review", "done"];
  if (issue.displayStatus === "blocked" || issue.displayStatus === "cancel") {
    states.push(issue.displayStatus);
  }
  const currentIndex = states.indexOf(issue.displayStatus);
  return states.map((state, index) => {
    const phase =
      state === "blocked" || state === "cancel"
        ? "exception"
        : currentIndex < 0
          ? "future"
          : index < currentIndex
            ? "past"
            : index === currentIndex
              ? "current"
              : "future";
    const events = phase === "future" ? [] : browserPreviewEventsForState(state, index, issue);
    return {
      state,
      phase,
      enteredAt: events.length ? previewTimestamp + index * 60 : null,
      events,
      summary: browserPreviewSummaryForState(state, issue),
      liveRefs: phase === "future" ? [] : browserPreviewLiveRefsForState(state, issue),
    };
  });
}

function browserPreviewEventsForState(state: TaskTimelineItem["state"], index: number, issue: InputIssue) {
  const eventTypes: Record<TaskTimelineItem["state"], string[]> = {
    backlog: ["issue.created"],
    blocked: ["issue.blocked"],
    cancel: ["issue.cancelled"],
    done: ["issue.pr.merged", "issue.completed"],
    in_progress: ["agent.launch.requested", "agent.session.running", "issue.validation.running"],
    in_review: ["issue.validation.passed", "issue.pr.created"],
    todo: ["issue.scheduled", "context-pack.ready", "workspace.clean"],
  };
  return (eventTypes[state] ?? []).map((eventType, eventIndex) => ({
    actorKind: eventType.startsWith("agent.") ? "agent" : "system",
    actorRole: eventType.startsWith("agent.") ? "build-agent" : "agentflow-loop",
    artifactRefs: eventIndex === 0 ? browserPreviewLiveRefsForState(state, issue).slice(0, 1) : [],
    eventId: `browser-${issue.issueId}-${state}-${eventIndex + 1}`,
    eventType,
    summary: browserPreviewEventSummary(eventType),
    timestamp: previewTimestamp + index * 60 + eventIndex * 12,
  }));
}

function browserPreviewEventSummary(eventType: string) {
  const summaries: Record<string, string> = {
    "agent.launch.requested": "已生成 Build Agent 启动请求。",
    "agent.session.running": "外部执行会话正在运行。",
    "context-pack.ready": "Context Pack 已就绪。",
    "issue.blocked": "任务进入阻断状态。",
    "issue.cancelled": "任务已取消。",
    "issue.completed": "任务 Done 写回完成。",
    "issue.created": "任务已生成。",
    "issue.pr.created": "PR/MR 已创建。",
    "issue.pr.merged": "PR/MR 已合并。",
    "issue.scheduled": "任务进入待执行队列。",
    "issue.validation.passed": "本地沙箱验证已通过。",
    "issue.validation.running": "正在运行本地沙箱验证。",
    "workspace.clean": "工作区已确认干净。",
  };
  return summaries[eventType] ?? `记录事件：${eventType}。`;
}

function browserPreviewLiveRefsForState(state: TaskTimelineItem["state"], issue: InputIssue) {
  const runId = previewRunIdForIssue(issue) ?? "run-browser-preview-pending";
  const refs: Record<TaskTimelineItem["state"], string[]> = {
    backlog: [issue.system?.path ?? `.agentflow/spec/issues/${issue.issueId}.json`],
    blocked: [".agentflow/events/task-events.jsonl"],
    cancel: [".agentflow/events/task-events.jsonl"],
    done: [`PR #100`, `CHANGELOG.md`, `.agentflow/tasks/${issue.issueId}/evidence/evidence.json`],
    in_progress: [
      `.agentflow/tasks/${issue.issueId}/runs/${runId}/run.json`,
      `.agentflow/tasks/${issue.issueId}/runs/${runId}/plan.json`,
      `.agentflow/tasks/${issue.issueId}/runs/${runId}/validate.log`,
    ],
    in_review: [`PR #100`, `.agentflow/tasks/${issue.issueId}/evidence/evidence.json`],
    todo: [issue.contextPackPath ?? `.agentflow/panel/context-packs/${issue.issueId}.json`],
  };
  return refs[state] ?? [];
}

function browserPreviewSummaryForState(state: TaskTimelineItem["state"], issue: InputIssue) {
  if (state === issue.displayStatus) {
    return issue.summary;
  }
  const summaries: Record<TaskTimelineItem["state"], string> = {
    backlog: "任务已生成，等待调度。",
    blocked: "任务被阻断，等待解除原因。",
    cancel: "任务已取消。",
    done: "任务完成，公开交付已留痕。",
    in_progress: "执行助手正在处理任务和本地验证。",
    in_review: "验证完成，等待 PR/MR 合并。",
    todo: "前置条件已满足，等待开工。",
  };
  return summaries[state] ?? "等待事件更新。";
}

function browserPreviewTransitionForState(state: TaskTimelineItem["state"]) {
  const transitions: Record<TaskTimelineItem["state"], string> = {
    backlog: "issue.created",
    blocked: "issue.blocked",
    cancel: "issue.cancelled",
    done: "issue.completed",
    in_progress: "agent.session.running",
    in_review: "issue.pr.created",
    todo: "issue.scheduled",
  };
  return transitions[state] ?? "issue.updated";
}

function previewRunIdForIssue(issue: InputIssue) {
  if (issue.displayStatus === "in_progress") {
    return previewProgressRunId;
  }
  if (issue.displayStatus === "in_review") {
    return previewReviewRunId;
  }
  if (issue.displayStatus === "done") {
    return previewDoneRunId;
  }
  return null;
}

export function createBrowserPreviewExecuteStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): ExecuteStatusSnapshot {
  const sessions = createBrowserPreviewMcpSessions(projectRoot, currentBrowserPreviewTaskHierarchyScenario());
  const activeRuns = sessions.filter((session) =>
    ["queued", "claimed", "starting", "running"].includes(session.status),
  ).length;
  const completedRuns = sessions.filter((session) =>
    ["in-review", "done"].includes(session.status),
  ).length;
  const blockedRuns = sessions.filter((session) =>
    ["failed", "cancelled"].includes(session.status),
  ).length;
  return {
    version: "execute-status.browser-preview",
    projectRoot,
    status: "ready",
    ready: true,
    manifestExists: true,
    indexExists: true,
    summary: {
      runs: sessions.length,
      activeRuns,
      blockedRuns,
      completedRuns,
      activeLeases: 0,
    },
    missingPaths: [],
    warnings: ["浏览器预览只展示模拟的执行状态，不执行命令，也不会应用补丁。"],
    errors: [],
  };
}

export function createBrowserPreviewMcpSessions(
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  scenario = currentBrowserPreviewTaskHierarchyScenario(),
): McpSessionSnapshot[] {
  if (scenario === "empty" || scenario === "empty-project" || scenario === "missing-issue") {
    return [];
  }

  const issues = browserPreviewIssuesForScenario(scenario);
  return issues
    .filter((issue) => ["in_progress", "in_review", "done"].includes(issue.displayStatus))
    .map((issue, index) => {
      const runId = previewRunIdForIssue(issue) ?? previewProgressRunId;
      const sessionId = `codex-${runId}`;
      const status =
        issue.displayStatus === "done"
          ? "done"
          : issue.displayStatus === "in_review"
            ? "in-review"
            : "running";
      return {
        version: "agentflow-mcp-session.browser-preview",
        provider: "codex",
        issueId: issue.issueId,
        projectId: issue.projectId ?? null,
        runId,
        sessionId,
        status,
        launchMode: "cli-exec-stdin",
        launchRequestPath: `.agentflow/tasks/${issue.issueId}/runs/${runId}/launch/agent-request.json`,
        planPath: `.agentflow/state/mcp/plans/${sessionId}.json`,
        logPath: `.agentflow/state/mcp/sessions/${sessionId}.jsonl`,
        branchName: `agentflow/browser-preview/${issue.issueId}`,
        pid: null,
        remoteSessionId: null,
        prUrl:
          status === "running"
            ? null
            : `https://github.com/atxinbao/agentflow/pull/${100 + index + 1}`,
        mergeState: status === "done" ? "merged" : status === "in-review" ? "open" : null,
        note: `${projectRoot} 浏览器预览执行会话`,
        lastError: null,
        createdAt: previewTimestamp - (index + 1) * 120,
        updatedAt: previewTimestamp - index * 60,
      } satisfies McpSessionSnapshot;
    })
    .sort((left, right) => right.updatedAt - left.updatedAt || right.sessionId.localeCompare(left.sessionId));
}

export function createBrowserPreviewOutputIndex(): OutputIndex {
  return {
    version: "output-index.browser-preview",
    updatedAt: previewTimestamp,
    evidence: [
      browserPreviewOutputEntry(
        previewReviewRunId,
        "iss-review",
        previewSpecId,
        `.agentflow/tasks/iss-review/evidence/evidence.json`,
        "complete",
      ),
      browserPreviewOutputEntry(
        previewDoneRunId,
        previewDoneIssueId,
        previewSpecId,
        `.agentflow/tasks/${previewDoneIssueId}/evidence/evidence.json`,
        "complete",
      ),
    ],
    audits: [
      browserPreviewOutputEntry(
        previewDoneRunId,
        previewDoneIssueId,
        previewSpecId,
        ".agentflow/audit/audit-browser-preview-001/audit-report.md",
        "passed-with-warnings",
      ),
    ],
  };
}

export function createBrowserPreviewAuditIndex(): AuditIndex {
  return {
    version: "audit-index.browser-preview",
    updatedAt: previewTimestamp,
    audits: [
      {
        auditId: previewAuditId,
        status: "passed-with-warnings",
        trigger: "human-via-agent",
        requestedBy: "agentflow-human-audit",
        requestedAt: previewTimestamp,
        sourceDeliveryId: previewDeliveryRunId,
        sourceRunId: previewDeliveryRunId,
        sourceIssueId: previewDoneIssueId,
        sourceSpecId: previewSpecId,
        reportPath: ".agentflow/audit/audit-browser-preview-001/audit-report.md",
        auditPath: ".agentflow/audit/audit-browser-preview-001/audit.json",
      },
    ],
  };
}

export function createBrowserPreviewHumanAuditReport(): HumanAuditReport | null {
  return {
    request: {
      trigger: "human-via-agent",
      reason: "审计请求已独立登记，用于核对交付材料。",
      source: {
        kind: "public-delivery",
        deliveryId: "CHANGELOG.md",
        runId: previewDeliveryRunId,
        issueId: previewDoneIssueId,
        specId: previewSpecId,
      },
      scope: {
        description: "交付关联审计，用于核对执行助手的交付结果。",
        refs: [
          {
            kind: "spec",
            id: previewSpecId,
            path: `docs/requirements/${previewSpecId}.md`,
          },
          {
            kind: "issue",
            id: previewDoneIssueId,
            path: `.agentflow/spec/issues/${previewDoneIssueId}.json`,
          },
          {
            kind: "task-run",
            id: previewDeliveryRunId,
            path: `.agentflow/tasks/${previewDoneIssueId}/runs/${previewDeliveryRunId}/`,
          },
          {
            kind: "evidence",
            id: previewDeliveryRunId,
            path: `.agentflow/tasks/${previewDoneIssueId}/evidence/evidence.json`,
          },
          {
            kind: "public-delivery",
            id: "CHANGELOG.md",
            path: "CHANGELOG.md",
          },
        ],
      },
    },
    audit: {
      auditId: previewAuditId,
      status: "passed-with-warnings",
      trigger: "human-via-agent",
      requestedBy: "agentflow-human-audit",
      requestedAt: previewTimestamp,
      sourceDeliveryId: previewDeliveryRunId,
      sourceRunId: previewDeliveryRunId,
      sourceIssueId: previewDoneIssueId,
      summary: {
        findings: 1,
        warnings: 1,
      },
      checks: {
        specAlignment: "passed",
        boundaryCompliance: "passed",
        evidenceCompleteness: "warning",
      },
      paths: {
        report: ".agentflow/audit/audit-browser-preview-001/audit-report.md",
      },
    },
    reportMarkdown:
      "# Human Audit Browser Preview\n\n" +
      "状态：通过，有警告。\n\n" +
      "- 已核对公开交付记录、execute run、evidence 和 issue scope refs。\n" +
      "- 浏览器预览只展示 mock 审计报告，不写 `.agentflow/audit`。\n",
    findings: [
      {
        id: "finding-browser-preview-001",
        severity: "warning",
        summary: "浏览器预览使用模拟审计包。",
      },
    ],
    checklistMarkdown:
      "- [x] Scope refs 自动生成\n" +
      "- [x] Request Human Audit 在浏览器预览中禁用\n" +
      "- [x] audit-report.md 可只读展示\n",
    evidenceMap: {
      evidence: [`.agentflow/tasks/${previewDoneIssueId}/evidence/evidence.json`],
      publicDelivery: ["CHANGELOG.md"],
    },
    traceability: {
      sourceSpecId: previewSpecId,
      issueId: previewDoneIssueId,
      runId: previewDeliveryRunId,
      auditId: previewAuditId,
    },
  };
}

function browserPreviewOutputEntry(
  runId: string,
  issueId: string,
  sourceSpecId: string,
  path: string,
  status: string,
): OutputIndexEntry {
  return {
    runId,
    issueId,
    sourceSpecId,
    path,
    status,
    updatedAt: previewTimestamp,
  };
}

export function createBrowserPreviewStateStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): StateStatusSnapshot {
  return {
    version: "state-status.browser-preview",
    projectRoot,
    status: "ready",
    currentStage: "workspace-ready",
    auditStatus: "passed-with-warnings",
    activeIssueId: null,
    activeRunId: null,
    health: {
      workspace: "ready",
      define: "ready",
      panel: "ready",
      input: "ready",
      execute: "ready",
      output: "ready",
      audit: "idle",
    },
    nextActions: ["start-new-input"],
    blockers: [],
    updatedAt: 1780600000,
  };
}

export function createBrowserPreviewPanelManifest(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): PanelManifestSnapshot {
  return {
    version: "panel-manifest.browser-preview",
    projectRoot,
    languages: ["markdown", "typescript", "rust", "toml", "json"],
    topLevelDirs: [".agentflow", ".git", "apps", "crates", "docs", "target"],
    importantFiles: ["README.md", "Cargo.toml", "design.md", "apps/desktop/package.json"],
    sourceFiles: 4,
    testFiles: 0,
    docFiles: 3,
    configFiles: 2,
    platforms: [],
    entryPoints: ["README.md", "Cargo.toml"],
    mobileComponents: [],
    mobileConfigs: [],
    mobileTests: [],
  };
}

export function createBrowserPreviewPanelSearch(query: string): PanelSearchSnapshot {
  const normalizedQuery = query.trim() || "project";
  return {
    version: "panel-search.browser-preview",
    query: normalizedQuery,
    results: [
      {
        kind: "file",
        path: "apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx",
        title: "ProjectLocalFilesPage.tsx",
        language: "typescript",
        symbolKind: "source",
        line: null,
        snippet: "Project 页面本地文件阅读器入口。",
        score: 0.88,
      },
      {
        kind: "symbol",
        path: "crates/panel/src/manager.rs",
        title: "prepare_project_panel",
        language: "rust",
        symbolKind: "function",
        line: 28,
        snippet: "准备本地 Project Panel 索引目录和状态。",
        score: 0.82,
      },
    ],
  };
}

export function createBrowserPreviewPanelContextPack(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): PanelContextPack {
  return {
    version: "panel-context-pack.browser-preview",
    targetType: "preview",
    targetId: "browser-preview",
    query: "Project 文件阅读器 Panel V1 浏览器预览",
    createdAt: previewTimestamp,
    panelRevision: "browser-preview",
    recommendedFiles: [
      {
        path: "apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx",
        reason: "Project 文件阅读器入口",
        score: 0.88,
      },
    ],
    recommendedSymbols: [
      {
        name: "ProjectLocalFilesPage",
        kind: "function",
        path: "apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx",
        line: 9,
        score: 0.82,
      },
    ],
    recommendedTests: [],
    impactHints: [
      {
        path: "apps/desktop/src/features/project-files/hooks/useProjectFiles.ts",
        reason: "同一 Project 文件读取链路",
        confidence: "medium",
      },
    ],
    testHints: [
      {
        commandHint: "npm --prefix apps/desktop run build",
        reason: "验证桌面前端类型和构建",
        confidence: "medium",
        scope: "package",
      },
    ],
    confidence: "medium",
  };
}

export function createBrowserPreviewProjectFileContent(relativePath: string, projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): ProjectFileContent {
  const normalizedPath = normalizeProjectRelativePath(relativePath || "README.md");
  const entry = findBrowserPreviewEntry(normalizedPath) ?? browserPreviewFileEntry(normalizedPath, "file");
  const content = browserPreviewFileContentByPath(normalizedPath, projectRoot);
  return {
    relativePath: normalizedPath,
    name: entry.name,
    kind: entry.kind,
    createdAt: entry.createdAt,
    modifiedAt: entry.modifiedAt,
    sizeBytes: entry.sizeBytes,
    extension: entry.extension,
    mimeType: content.mimeType,
    language: content.language,
    content: entry.kind === "file" ? content.content : null,
    binaryPreview: content.binaryPreview,
    dataUrl: null,
    truncated: false,
    directoryChildren: entry.kind === "directory" ? entry.children : [],
    unsupportedReason: null,
  };
}

export function createBrowserPreviewProjectFileTextRange(
  relativePath: string,
  startLine: number,
  lineCount: number,
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
): ProjectFileTextRange {
  const content = createBrowserPreviewProjectFileContent(relativePath, projectRoot);
  const lines = (content.content ?? "").split("\n");
  const safeStartLine = Math.max(1, Math.floor(startLine || 1));
  const safeLineCount = Math.max(1, Math.floor(lineCount || 240));
  const startIndex = Math.min(safeStartLine - 1, lines.length);
  const endIndex = Math.min(startIndex + safeLineCount, lines.length);
  return {
    version: "project-file-text-range.v1",
    projectRoot,
    relativePath: content.relativePath,
    startLine: safeStartLine,
    endLine: endIndex,
    totalLines: lines.length,
    content: lines.slice(startIndex, endIndex).join("\n"),
    truncated: endIndex < lines.length,
  };
}

export function createBrowserPreviewProjectDirectoryPage(
  directoryPath: string,
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  viewMode: ProjectFileViewMode | string = "source",
  cursor?: string | null,
): ProjectDirectoryPage {
  const normalizedPath = normalizeProjectRelativePath(directoryPath);
  const offset = cursor ? Number.parseInt(cursor, 10) || 0 : 0;
  const limit = 80;
  const allEntries = filterBrowserPreviewChildren(browserPreviewDirectoryChildren(normalizedPath), viewMode);
  const entries = allEntries.slice(offset, offset + limit);
  const nextOffset = offset + entries.length;
  return {
    version: "project-directory-page.browser-preview",
    projectRoot,
    directoryPath: normalizedPath,
    entries,
    nextCursor: nextOffset < allEntries.length ? String(nextOffset) : null,
    totalChildren: allEntries.length,
    limit,
    viewMode,
  };
}

export function createBrowserPreviewProjectFileSearchSnapshot(
  query: string,
  projectRoot = BROWSER_PREVIEW_PROJECT_ROOT,
  viewMode: ProjectFileViewMode | string = "source",
): ProjectFileSearchSnapshot {
  const normalizedQuery = query.trim().toLowerCase();
  const results = flattenBrowserPreviewEntries(filterBrowserPreviewEntries(browserPreviewTopLevelEntries(), viewMode))
    .filter((entry) => {
      const path = entry.relativePath.toLowerCase();
      const name = entry.name.toLowerCase();
      return name.includes(normalizedQuery) || path.includes(normalizedQuery);
    })
    .slice(0, 80)
    .map((entry) => ({
      name: entry.name,
      relativePath: entry.relativePath,
      kind: entry.kind,
      extension: entry.extension,
      modifiedAt: entry.modifiedAt,
      sizeBytes: entry.sizeBytes,
      score: entry.name.toLowerCase().startsWith(normalizedQuery) ? 100 : 60,
      matchReason: entry.name.toLowerCase().includes(normalizedQuery) ? "name" : "path",
    }));
  return {
    version: "project-file-search.browser-preview",
    projectRoot,
    query,
    viewMode,
    results,
  };
}

function browserPreviewTopLevelEntries(): ProjectFileEntry[] {
  return [
    browserPreviewDirectoryEntry(".git", [browserPreviewFileChild(".git/HEAD"), browserPreviewFileChild(".git/config")]),
    browserPreviewFileEntry(".DS_Store", "file", 6148),
    browserPreviewFileEntry(".gitignore", "file", 128),
    browserPreviewFileEntry("Cargo.toml", "file", 640),
    browserPreviewFileEntry("README.md", "file", 1280),
    browserPreviewFileEntry("design.md", "file", 2200),
    browserPreviewDirectoryEntry("apps", [browserPreviewDirectoryChild("apps/desktop")]),
    browserPreviewDirectoryEntry("crates", [browserPreviewDirectoryChild("crates/core")]),
    browserPreviewDirectoryEntry("docs", [browserPreviewDirectoryChild("docs/requirements")]),
    browserPreviewDirectoryEntry("target", []),
  ];
}

function browserPreviewDirectoryChildren(path: string): ProjectFileChild[] {
  return browserPreviewDirectoryChildSpecs(path).map((child) =>
    child.kind === "directory" ? browserPreviewDirectoryChild(child.relativePath) : browserPreviewFileChild(child.relativePath),
  );
}

function findBrowserPreviewEntry(relativePath: string): ProjectFileEntry | null {
  const normalizedPath = normalizeProjectRelativePath(relativePath);
  const topLevelEntries = browserPreviewTopLevelEntries();
  const directTopLevel = topLevelEntries.find((entry) => entry.relativePath === normalizedPath);
  if (directTopLevel) {
    return directTopLevel;
  }
  const name = normalizedPath.split("/").at(-1) ?? normalizedPath;
  const isDirectory = browserPreviewDirectoryPathSet.has(normalizedPath);
  return {
    name,
    relativePath: normalizedPath,
    kind: isDirectory ? "directory" : "file",
    createdAt: previewTimestamp,
    modifiedAt: previewTimestamp,
    sizeBytes: isDirectory ? null : browserPreviewFileContentByPath(normalizedPath, BROWSER_PREVIEW_PROJECT_ROOT).content.length,
    extension: isDirectory ? null : getProjectFileExtensionFromName(name),
    childCount: isDirectory ? browserPreviewDirectoryChildSpecs(normalizedPath).length : null,
    children: isDirectory ? browserPreviewDirectoryChildren(normalizedPath) : [],
  };
}

function browserPreviewDirectoryEntry(relativePath: string, children: ProjectFileChild[]): ProjectFileEntry {
  const name = relativePath.split("/").at(-1) ?? relativePath;
  return {
    name,
    relativePath,
    kind: "directory",
    createdAt: previewTimestamp,
    modifiedAt: previewTimestamp,
    sizeBytes: null,
    extension: null,
    childCount: children.length,
    isSymlink: false,
    children,
  };
}

function browserPreviewFileEntry(relativePath: string, kind: "file", sizeBytes?: number): ProjectFileEntry {
  const name = relativePath.split("/").at(-1) ?? relativePath;
  return {
    name,
    relativePath,
    kind,
    createdAt: previewTimestamp,
    modifiedAt: previewTimestamp,
    sizeBytes: sizeBytes ?? 512,
    extension: getProjectFileExtensionFromName(name),
    childCount: null,
    isSymlink: false,
    children: [],
  };
}

function browserPreviewDirectoryChild(relativePath: string): ProjectFileChild {
  const name = relativePath.split("/").at(-1) ?? relativePath;
  return {
    name,
    relativePath,
    kind: "directory",
    createdAt: previewTimestamp,
    modifiedAt: previewTimestamp,
    sizeBytes: null,
    extension: null,
    childCount: browserPreviewDirectoryChildSpecs(relativePath).length,
    isSymlink: false,
  };
}

const browserPreviewDirectoryPathSet = new Set([
  ".git",
  "apps",
  "apps/desktop",
  "apps/desktop/src",
  "apps/desktop/src/features",
  "apps/desktop/src/features/project-files",
  "crates",
  "crates/core",
  "crates/core/src",
  "docs",
  "docs/requirements",
  "target",
  "target/debug",
]);

function browserPreviewDirectoryChildSpecs(path: string): Array<{ relativePath: string; kind: "directory" | "file" }> {
  const childrenByPath: Record<string, Array<{ relativePath: string; kind: "directory" | "file" }>> = {
    ".git": [
      { relativePath: ".git/HEAD", kind: "file" },
      { relativePath: ".git/config", kind: "file" },
    ],
    apps: [{ relativePath: "apps/desktop", kind: "directory" }],
    "apps/desktop": [
      { relativePath: "apps/desktop/package.json", kind: "file" },
      { relativePath: "apps/desktop/src", kind: "directory" },
    ],
    "apps/desktop/src": [
      { relativePath: "apps/desktop/src/App.tsx", kind: "file" },
      { relativePath: "apps/desktop/src/features", kind: "directory" },
    ],
    "apps/desktop/src/features": [{ relativePath: "apps/desktop/src/features/project-files", kind: "directory" }],
    "apps/desktop/src/features/project-files": [
      { relativePath: "apps/desktop/src/features/project-files/hooks/useProjectFiles.ts", kind: "file" },
      { relativePath: "apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx", kind: "file" },
    ],
    crates: [{ relativePath: "crates/core", kind: "directory" }],
    "crates/core": [{ relativePath: "crates/core/src", kind: "directory" }],
    "crates/core/src": [{ relativePath: "crates/core/src/lib.rs", kind: "file" }],
    docs: [{ relativePath: "docs/requirements", kind: "directory" }],
    "docs/requirements": [{ relativePath: "docs/requirements/001-add-local-project.md", kind: "file" }],
    target: [{ relativePath: "target/debug", kind: "directory" }],
    "target/debug": [],
  };
  return childrenByPath[path] ?? [];
}

function browserPreviewFileChild(relativePath: string): ProjectFileChild {
  const name = relativePath.split("/").at(-1) ?? relativePath;
  return {
    name,
    relativePath,
    kind: "file",
    createdAt: previewTimestamp,
    modifiedAt: previewTimestamp,
    sizeBytes: browserPreviewFileContentByPath(relativePath, BROWSER_PREVIEW_PROJECT_ROOT).content.length,
    extension: getProjectFileExtensionFromName(name),
    childCount: null,
    isSymlink: false,
  };
}

function filterBrowserPreviewEntries(entries: ProjectFileEntry[], viewMode: ProjectFileViewMode | string) {
  if (viewMode === "all") {
    return entries;
  }
  return entries.filter((entry) => !browserPreviewSourceExcluded(entry.relativePath));
}

function filterBrowserPreviewChildren(children: ProjectFileChild[], viewMode: ProjectFileViewMode | string) {
  if (viewMode === "all") {
    return children;
  }
  return children.filter((entry) => !browserPreviewSourceExcluded(entry.relativePath));
}

function browserPreviewSourceExcluded(relativePath: string) {
  return relativePath
    .split("/")
    .some((part) => [".git", ".agentflow", ".codex", "target", "node_modules", "dist", "build", "agent-artifacts"].includes(part));
}

function flattenBrowserPreviewEntries(entries: ProjectFileEntry[]): ProjectFileEntry[] {
  const result: ProjectFileEntry[] = [];
  function visit(entry: ProjectFileEntry) {
    result.push(entry);
    entry.children.forEach((child) => {
      const childEntry = findBrowserPreviewEntry(child.relativePath);
      if (childEntry) {
        visit(childEntry);
      }
    });
  }
  entries.forEach(visit);
  return result;
}

function browserPreviewFileContentByPath(relativePath: string, projectRoot: string) {
  const contentByPath: Record<string, { content: string; language: string; mimeType: string | null; binaryPreview?: string | null }> = {
    "README.md": {
      language: "markdown",
      mimeType: "text/markdown",
      content: `# AgentFlow\n\n浏览器预览模式使用这份 mock 项目数据来验证 Desktop UI。\n\n## 边界\n\n- 真实桌面客户端读取 ${projectRoot} 下的本地文件。\n- 浏览器预览不具备 Tauri 本地命令能力，因此只展示 mock 文件树。\n- 浏览器预览不会写入 .agentflow/、.codex/ 或 agent-artifacts/。\n`,
    },
    "design.md": {
      language: "markdown",
      mimeType: "text/markdown",
      content: "# AgentFlow Project Page Design\n\nProject 页面是本地项目文件阅读器。\n\n- 左侧：固定项目导航。\n- 顶部：当前项目名称和路径。\n- 主体：文件内容阅读器。\n- 右侧：Finder 风格文件列表。\n",
    },
    "Cargo.toml": {
      language: "toml",
      mimeType: "text/plain",
      content: '[workspace]\nmembers = ["crates/core", "apps/desktop/src-tauri"]\nresolver = "2"\n',
    },
    ".gitignore": {
      language: "config",
      mimeType: "text/plain",
      content: "target/\nnode_modules/\ndist/\n.agentflow/\n.DS_Store\n",
    },
    ".DS_Store": {
      language: "binary",
      mimeType: "application/octet-stream",
      content: "",
      binaryPreview: "00000000  00 05 16 07 00 02 00 00 4d 6f 63 6b 20 44 53 20",
    },
    ".git/HEAD": {
      language: "config",
      mimeType: "text/plain",
      content: "ref: refs/heads/main\n",
    },
    ".git/config": {
      language: "config",
      mimeType: "text/plain",
      content: "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n",
    },
    "apps/desktop/package.json": {
      language: "json",
      mimeType: "application/json",
      content: '{\n  "name": "agentflow-desktop",\n  "private": true,\n  "scripts": {\n    "build": "tsc && vite build",\n    "dev": "vite --host 127.0.0.1"\n  }\n}\n',
    },
    "apps/desktop/src/App.tsx": {
      language: "typescript",
      mimeType: "text/plain",
      content: 'import { ProjectLocalFilesPage } from "./features/project-files";\n\nexport function AppPreviewNote() {\n  return "Browser preview uses explicit mock data only outside Tauri.";\n}\n',
    },
    "apps/desktop/src/features/project-files/hooks/useProjectFiles.ts": {
      language: "typescript",
      mimeType: "text/plain",
      content: 'export function isBrowserPreviewRuntime() {\n  return typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window);\n}\n',
    },
    "apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx": {
      language: "typescript",
      mimeType: "text/plain",
      content: "export function ProjectLocalFilesPage() {\n  return null;\n}\n",
    },
    "crates/core/src/lib.rs": {
      language: "rust",
      mimeType: "text/plain",
      content: "pub fn agentflow_preview_boundary() -> &'static str {\n    \"browser preview is read-only\"\n}\n",
    },
    "docs/requirements/001-add-local-project.md": {
      language: "markdown",
      mimeType: "text/markdown",
      content: "# Add Local Project\n\n浏览器预览可以使用 mock 数据验证 UI；真实 Tauri 客户端必须读取真实本地项目。\n",
    },
  };

  return (
    contentByPath[relativePath] ?? {
      language: getProjectFileExtensionFromName(relativePath) || "text",
      mimeType: "text/plain",
      content: `# ${relativePath}\n\n这是浏览器预览 mock 内容。真实客户端会读取本地文件系统。`,
    }
  );
}
