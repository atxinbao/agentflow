import type {
  PanelContextPack,
  PanelManifestSnapshot,
  PanelSearchSnapshot,
  PanelStatusSnapshot,
  IssueContract,
  LocalMetricsSnapshot,
  LocalProjectModelSnapshot,
  LocalSearchSnapshot,
  ProjectDirectoryPage,
  ProjectFileChild,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileSearchSnapshot,
  ProjectFileTextRange,
  ProjectFileViewMode,
  ProjectFilesSnapshot,
  ProjectMilestoneIssueViewModelSnapshot,
  WorkbenchBoundary,
  WorkbenchSnapshot,
  AgentEnvironmentStatus,
  InputIssue,
  InputSnapshot,
  InputStatusSnapshot,
  IssueStatusIndex,
  ExecuteStatusSnapshot,
  OutputStatusSnapshot,
  OutputIndex,
  OutputIndexEntry,
  AuditIndex,
  HumanAuditReport,
  StateStatusSnapshot,
} from "./types";
import {
  getProjectFileExtensionFromName,
  normalizeProjectRelativePath,
} from "./features/project-files/model/projectFileUtils";

export const BROWSER_PREVIEW_PROJECT_ROOT = "/Users/mac/Documents/AgentFlow";

const previewBoundary: WorkbenchBoundary = {
  readOnly: true,
  disallowedActions: ["不执行命令", "不写入项目文件", "不调用模型", "不创建远程对象"],
};

const previewTimestamp = 1780291200;
const previewIssueId = "iss-001";
const previewAuditId = "audit-browser-preview-001";
const previewDeliveryRunId = "run-browser-preview-001";
const previewProjectId = "project-browser-preview";
const previewSpecId = "spec-browser-preview";

const previewIssueContract: IssueContract = {
  id: "ISSUE-PREVIEW-001",
  title: "浏览器预览文件阅读器",
  status: "todo",
  intent: "验证浏览器预览环境下的项目文件阅读器、文件列表和只读边界。",
  scope: ["展示浏览器预览专用文件树。", "展示 Markdown、配置文件、代码和目录概览。", "保持真实桌面客户端只读取真实本地文件。"],
  nonGoals: ["不执行命令。", "不写入本地工作区。", "不调用模型。", "不创建远程对象。"],
  context: {
    repo: BROWSER_PREVIEW_PROJECT_ROOT,
    files: ["apps/desktop/src/App.tsx", "apps/desktop/src/features/project-files/hooks/useProjectFiles.ts"],
  },
  executionPlan: ["在浏览器预览中加载 mock 文件树。", "点击文件后在阅读器展示 mock 内容。", "真实 Tauri 客户端仍通过本地命令读取文件。"],
  validation: {
    commands: ["npm --prefix apps/desktop run build", "cargo test", "git diff --check"],
  },
  evidenceRequirements: ["浏览器预览可展示项目文件列表。", "真实客户端不使用 mock fallback。", "无法读取本地文件时不暴露 raw invoke 错误。"],
  humanGate: {
    beforeExternalNetwork: true,
    beforeFileEdits: true,
  },
};

const previewInputIssues: InputIssue[] = [
  browserPreviewInputIssue("iss-backlog", "整理需求入口", "planned", "backlog", "需求已创建，等待整理成 SPEC。"),
  browserPreviewInputIssue("iss-ready", "生成执行任务包", "ready-for-execute", "ready", "SPEC 已确认，可以交给 Agent。", {
    blocks: ["iss-progress"],
    issueModel: "project",
    projectId: previewProjectId,
  }),
  browserPreviewInputIssue("iss-progress", "执行受控改动", "ready-for-execute", "in-progress", "Agent 已接手任务。", {
    blockedBy: ["iss-ready"],
    blocks: ["iss-review"],
    issueModel: "project",
    projectId: previewProjectId,
    riskLevel: "medium",
  }),
  browserPreviewInputIssue("iss-review", "审计交付材料", "ready-for-execute", "review", "任务已交付，等待人工审计。", {
    blockedBy: ["iss-progress"],
    issueModel: "project",
    projectId: previewProjectId,
  }),
  browserPreviewInputIssue("iss-audit-ready", "复制审计任务包", "ready-for-execute", "ready", "审计任务可以交给 Audit Agent。", {
    audit: {
      auditId: previewAuditId,
      auditOutputDir: `.agentflow/output/audit/${previewAuditId}`,
      expectedOutputs: previewAuditExpectedOutputs(previewAuditId),
      sourceDeliveryPath: `.agentflow/output/release/${previewDeliveryRunId}/delivery.json`,
      sourceReleaseId: previewDeliveryRunId,
      trigger: "manual",
    },
    issueCategory: "audit",
    issueModel: "project",
    projectId: previewProjectId,
    requiredAgentRole: "audit-agent",
  }),
  browserPreviewInputIssue("iss-done", "确认交付完成", "done", "done", "审计已通过。", {
    issueModel: "project",
    projectId: previewProjectId,
  }),
  browserPreviewInputIssue("iss-cancel", "取消过期需求", "canceled", "cancel", "任务已取消。"),
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
    requiredAgentRole?: InputIssue["requiredAgentRole"];
    riskLevel?: string;
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
    priority: "normal",
    status,
    displayStatus,
    riskLevel: options.riskLevel ?? "low",
    expectedOutputs: options.expectedOutputs ?? (issueCategory === "spec" ? previewBuildExpectedOutputs(issueId) : undefined),
    scope: previewIssueContract.scope,
    nonGoals: previewIssueContract.nonGoals,
    acceptanceCriteria: previewIssueContract.evidenceRequirements,
    validationHints: previewIssueContract.validation.commands,
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
      path: `.agentflow/input/issues/${issueId}.json`,
      revision: 1,
    },
  };
}

function previewBuildExpectedOutputs(issueId: string) {
  return {
    evidencePath: `.agentflow/output/evidence/${issueId}.json`,
    executeRunDir: `.agentflow/execute/runs/${issueId}`,
    releaseDeliveryDir: `.agentflow/output/release/${issueId}`,
  };
}

function previewAuditExpectedOutputs(auditId: string) {
  const outputDir = `.agentflow/output/audit/${auditId}`;
  return {
    "audit-report.md": `${outputDir}/audit-report.md`,
    "audit.json": `${outputDir}/audit.json`,
    "evidence-map.json": `${outputDir}/evidence-map.json`,
    "findings.json": `${outputDir}/findings.json`,
    "traceability.json": `${outputDir}/traceability.json`,
  };
}

export function createBrowserPreviewWorkbenchSnapshot(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): WorkbenchSnapshot {
  return {
    version: "workbench.browser-preview",
    initialized: true,
    projectRoot,
    projectSummaryMarkdown: "# AgentFlow 浏览器预览\n\n用于在浏览器中验证 Desktop 文件阅读器和项目结构展示。",
    goalLoopSummaryMarkdown: null,
    goalLoop: {
      version: "goal-loop.browser-preview",
      goalReady: true,
      activeIssueId: previewIssueContract.id,
      incompleteIssues: [
        {
          id: previewIssueContract.id,
          title: previewIssueContract.title,
          status: previewIssueContract.status,
          nextAction: "浏览器预览验证",
        },
      ],
      nextAction: "浏览器预览验证",
      recommendedIssueIntent: previewIssueContract.intent,
      recommendedCommand: "npm --prefix apps/desktop run build",
      rationale: ["浏览器预览使用显式 mock 数据；真实桌面客户端仍读取本地文件。"],
      counts: {
        issues: 1,
        completedIssues: 0,
        runs: 0,
        evidenceReports: 0,
        reviews: 0,
        projectUpdates: 0,
      },
      sources: {
        preview: "apps/desktop/src/browserPreviewData.ts",
      },
    },
    issues: [previewIssueContract],
    runs: [],
    savedViews: [
      {
        version: "saved-view.browser-preview",
        id: "view-preview-files",
        name: "浏览器预览文件",
        filter: {
          issueStatus: "todo",
          runStatus: null,
          validationStatus: null,
          issueId: previewIssueContract.id,
        },
      },
    ],
    evidence: [],
    reviews: [],
    projectUpdates: [],
    counts: {
      issues: 1,
      completedIssues: 0,
      runs: 0,
      passedRuns: 0,
      evidenceReports: 0,
      reviews: 0,
      projectUpdates: 0,
      savedViews: 1,
    },
    boundary: previewBoundary,
  };
}

export function createBrowserPreviewMetricsSnapshot(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): LocalMetricsSnapshot {
  return {
    version: "metrics.browser-preview",
    initialized: true,
    projectRoot,
    issues: {
      total: 1,
      completed: 0,
      planned: 1,
      active: 0,
    },
    runs: {
      total: 0,
      passed: 0,
      failed: 0,
      missingValidation: 0,
    },
    artifacts: {
      evidenceReports: 0,
      reviews: 0,
      projectUpdates: 0,
      savedViews: 1,
    },
    goalReady: true,
    activeIssueId: previewIssueContract.id,
    nextAction: "浏览器预览验证",
    recommendedCommand: "npm --prefix apps/desktop run build",
    sources: ["apps/desktop/src/browserPreviewData.ts"],
    boundary: previewBoundary,
  };
}

export function createBrowserPreviewProjectModelSnapshot(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): LocalProjectModelSnapshot {
  return {
    version: "project-model.browser-preview",
    initialized: true,
    projectRoot,
    workspace: {
      version: "workspace.browser-preview",
      id: "workspace-browser-preview",
      name: "浏览器预览工作区",
      defaultTeamId: "core",
      activeProjectId: "agentflow-browser-preview",
      teamIds: ["core"],
      projectIds: ["agentflow-browser-preview"],
      issueCount: 1,
      completedIssueCount: 0,
    },
    teams: [
      {
        version: "team.browser-preview",
        id: "core",
        name: "Core",
        workflow: ["define", "execute", "output"],
        defaultValidationCommands: ["npm --prefix apps/desktop run build"],
        wipLimit: 1,
        issueIds: [previewIssueContract.id],
      },
    ],
    projects: [
      {
        version: "project.browser-preview",
        id: "agentflow-browser-preview",
        name: "AgentFlow",
        status: "active",
        canonicalStatus: "active",
        goal: "验证浏览器预览环境下的项目文件阅读器。",
        teamIds: ["core"],
        activeMilestoneId: "milestone-browser-preview",
        milestones: [
          {
            id: "milestone-browser-preview",
            name: "浏览器预览",
            description: "验证 mock 文件树和文件阅读器。",
            sortOrder: 1,
            target: "Desktop browser preview",
            status: "active",
            progress: {
              doneIssueCount: 0,
              totalIssueCount: 1,
              nonCanceledIssueCount: 1,
              canceledIssueCount: 0,
              percent: 0,
            },
            issueIds: [previewIssueContract.id],
            completedIssueIds: [],
            nextIssueIntent: previewIssueContract.intent,
          },
        ],
        issueIds: [previewIssueContract.id],
        issueCount: 1,
        completedIssueCount: 0,
        nextIssueIntent: previewIssueContract.intent,
        recommendedCommand: "npm --prefix apps/desktop run build",
      },
    ],
    issueRefs: [
      {
        id: previewIssueContract.id,
        title: previewIssueContract.title,
        status: "todo",
        canonicalStatus: "todo",
        nextAction: "浏览器预览验证",
        latestRunId: null,
        latestRunStatus: null,
        validationStatus: "not_run",
        executionState: "ready",
        evidencePath: null,
        reviewPath: null,
        projectUpdatePath: null,
      },
    ],
    goalLoopSelection: {
      activeProjectId: "agentflow-browser-preview",
      source: "browser-preview",
      nextAction: "浏览器预览验证",
      nextIssueIntent: previewIssueContract.intent,
      recommendedCommand: "npm --prefix apps/desktop run build",
      rationale: ["浏览器预览使用显式 mock 数据，便于 UI 验证。"],
    },
    sources: ["apps/desktop/src/browserPreviewData.ts"],
    boundary: previewBoundary,
  };
}

export function createBrowserPreviewProjectViewModelSnapshot(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): ProjectMilestoneIssueViewModelSnapshot {
  return {
    version: "project-view-model.browser-preview",
    initialized: true,
    projectRoot,
    workspace: {
      id: "workspace-browser-preview",
      name: "浏览器预览工作区",
      activeProjectId: "agentflow-browser-preview",
      teamIds: ["core"],
      projectIds: ["agentflow-browser-preview"],
    },
    teams: [
      {
        id: "core",
        name: "Core",
        projectIds: ["agentflow-browser-preview"],
        issueIds: [previewIssueContract.id],
      },
    ],
    projects: [
      {
        id: "agentflow-browser-preview",
        name: "AgentFlow",
        status: "active",
        rawStatus: "active",
        goal: "验证浏览器预览环境下的项目文件阅读器。",
        targetMaturity: "MVP",
        targetLayers: ["Desktop", "Project Files"],
        scope: previewIssueContract.scope,
        nonGoals: previewIssueContract.nonGoals,
        successCriteria: ["浏览器预览可展示 mock 文件树。", "真实客户端仍使用 Tauri 命令读取本地文件。"],
        milestones: [
          {
            id: "milestone-browser-preview",
            projectId: "agentflow-browser-preview",
            name: "浏览器预览",
            status: "active",
            rawStatus: "active",
            goal: "验证 mock 文件树和文件阅读器。",
            entryCriteria: ["打开 http://127.0.0.1:1420/。"],
            scope: ["浏览器预览 UI。"],
            nonGoals: ["不写入真实工作区。"],
            issueIds: [previewIssueContract.id],
            exitCriteria: ["页面可展示文件列表和文件内容。"],
            validation: ["npm --prefix apps/desktop run build"],
            evidenceRequired: ["浏览器 smoke 结果。"],
            nextMilestoneGate: "真实客户端继续读取真实文件。",
            progress: {
              doneIssueCount: 0,
              totalIssueCount: 1,
              nonCanceledIssueCount: 1,
              canceledIssueCount: 0,
              percent: 0,
            },
          },
        ],
        issueOrder: [previewIssueContract.id],
        validationGate: previewIssueContract.validation.commands,
        evidenceRequired: previewIssueContract.evidenceRequirements,
        queueRule: ["浏览器预览不执行任务。"],
        closureGate: [],
      },
    ],
    issues: [
      {
        id: previewIssueContract.id,
        projectId: "agentflow-browser-preview",
        milestoneId: "milestone-browser-preview",
        title: previewIssueContract.title,
        issueCategory: "spec",
        requiredAgentRole: "build-agent",
        sourceSpecId: "browser-preview-spec",
        sourceSpecPath: ".agentflow/input/specs/approved/browser-preview-spec/spec.json",
        issuePath: `.agentflow/input/issues/${previewIssueContract.id}.json`,
        handoffId: `handoff-${previewIssueContract.id}`,
        contextPackPath: null,
        status: "todo",
        rawStatus: "todo",
        goal: previewIssueContract.intent,
        scope: previewIssueContract.scope,
        nonGoals: previewIssueContract.nonGoals,
        dependencies: [],
        codexInstructions: previewIssueContract.executionPlan,
        acceptanceCriteria: ["浏览器预览可展示 mock 文件内容。", "真实客户端不使用 mock fallback。"],
        validationCommands: previewIssueContract.validation.commands,
        expectedOutputs: {
          executeRunDir: `.agentflow/execute/runs/${previewIssueContract.id}`,
          evidencePath: `.agentflow/output/evidence/${previewIssueContract.id}.json`,
          releaseDeliveryDir: `.agentflow/output/release/${previewIssueContract.id}`,
        },
        evidenceRequired: previewIssueContract.evidenceRequirements,
        allowedFiles: previewIssueContract.context.files,
        forbiddenFiles: [".agentflow/*", ".codex/*", "agent-artifacts/*"],
        forbiddenActions: ["process-audit-issue", "write-audit-report", "write-audit-findings"],
        boundary: previewIssueContract.nonGoals,
        riskLevel: "low",
      },
    ],
    views: [
      {
        id: "view-browser-preview",
        name: "浏览器预览任务",
        entity: "issue",
        filter: {
          issueStatus: "todo",
          runStatus: null,
          validationStatus: null,
          issueId: previewIssueContract.id,
        },
        sort: [{ field: "id", direction: "asc" }],
        layout: "list",
      },
    ],
    invariants: ["浏览器预览可使用 mock 数据。", "真实 Tauri 客户端不能使用 mock fallback。"],
    sources: ["apps/desktop/src/browserPreviewData.ts"],
    boundary: previewBoundary,
  };
}

export function createBrowserPreviewSearchSnapshot(query: string, projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): LocalSearchSnapshot {
  return {
    version: "search.browser-preview",
    initialized: true,
    projectRoot,
    query: { query },
    searchedPaths: ["README.md", "apps/desktop/src/App.tsx", "apps/desktop/src/features/project-files/hooks/useProjectFiles.ts"],
    excludedPaths: [],
    results: [
      {
        sourceType: "browser-preview",
        entityKind: "file",
        entityId: null,
        path: "README.md",
        title: "README.md",
        field: "content",
        line: 1,
        snippet: `浏览器预览 mock 搜索结果：${query}`,
        score: 1,
      },
      {
        sourceType: "browser-preview",
        entityKind: "issue",
        entityId: previewIssueContract.id,
        path: "apps/desktop/src/browserPreviewData.ts",
        title: previewIssueContract.title,
        field: "intent",
        line: 1,
        snippet: previewIssueContract.intent,
        score: 0.82,
      },
    ],
    boundary: previewBoundary,
  };
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
    degradedReasons: [],
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
    warnings: ["Browser Preview shows mock Agent Manual state and does not write AGENTS.md."],
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
        ".agentflow/input",
        ".agentflow/input/manifest.json",
        ".agentflow/input/index.json",
        ".agentflow/panel",
        ".agentflow/execute",
        ".agentflow/output",
        ".agentflow/output/release",
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
    legacyAgentEntry: {
      exists: false,
      path: "AGENT.MD",
      managed: false,
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

export function createBrowserPreviewInputStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): InputStatusSnapshot {
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
      projects: 1,
      issues: previewInputIssues.length,
      blockedIssues: 0,
      highRiskIssues: 0,
    },
    missingPaths: [],
    warnings: ["浏览器预览只展示 mock input 状态，不写 .agentflow/input。"],
    errors: [],
  };
}

export function createBrowserPreviewInputSnapshot(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): InputSnapshot {
  return {
    version: "input-snapshot.browser-preview",
    projectRoot,
    ready: true,
    status: createBrowserPreviewInputStatus(projectRoot),
    manifest: {
      version: "input-manifest.browser-preview",
      projectRoot,
      status: "ready",
    },
    index: {
      version: "input-index.browser-preview",
      issues: previewInputIssues.map((issue) => ({
        id: issue.issueId,
        title: issue.title,
        path: issue.system?.path ?? `.agentflow/input/issues/${issue.issueId}.json`,
        status: issue.status,
        displayStatus: issue.displayStatus,
      })),
    },
    intake: [],
    specs: [],
    projects: [
      {
        version: "input-project.browser-preview",
        projectId: previewProjectId,
        sourceSpecId: previewSpecId,
        title: "浏览器预览任务项目",
        summary: "用于验证 Project Summary 和 Issue Contract 阅读器。",
        objective: "在任务页展示项目分组、推荐任务、依赖摘要和 issue 合约。",
        scope: ["展示项目摘要。", "展示 issue 所属项目和输出目标。", "验证建议任务按钮会切换到 issue 合约。"],
        nonGoals: ["不写入真实 .agentflow/input。", "不创建远程对象。"],
        successCriteria: ["项目行可选中。", "右侧可显示 Project Summary。", "查看建议任务后右侧显示 Issue Contract。"],
        issueIds: ["iss-ready", "iss-progress", "iss-review", "iss-audit-ready", "iss-done"],
        status: "active",
        panel: {
          snapshotId: null,
          contextPackId: null,
        },
        system: {
          createdBy: "browser-preview",
          createdAt: previewTimestamp,
          updatedAt: previewTimestamp,
          path: `.agentflow/input/projects/${previewProjectId}.json`,
          revision: 1,
        },
      },
    ],
    issues: previewInputIssues,
    relations: {
      version: "input-issue-relations.browser-preview",
      relations: [
        { fromIssueId: "iss-progress", toIssueId: "iss-ready", type: "blocked-by" },
        { fromIssueId: "iss-progress", toIssueId: "iss-review", type: "blocks" },
      ],
    },
  };
}

export function createBrowserPreviewIssueStatusIndex(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): IssueStatusIndex {
  return {
    version: "state-issue-status-index.browser-preview",
    updatedAt: previewTimestamp,
    issues: previewInputIssues.map((issue) => ({
      issueId: issue.issueId,
      displayStatus: issue.displayStatus,
      riskLevel: issue.riskLevel,
      latestRunId: issue.displayStatus === "in-progress" || issue.displayStatus === "review" || issue.displayStatus === "done" ? previewDeliveryRunId : null,
      executeStatus:
        issue.displayStatus === "in-progress"
          ? "running"
          : issue.displayStatus === "review" || issue.displayStatus === "done"
            ? "completed"
            : null,
      evidenceStatus: issue.displayStatus === "review" || issue.displayStatus === "done" ? "complete" : "missing",
      deliveryStatus: issue.displayStatus === "review" || issue.displayStatus === "done" ? "drafted" : "missing",
      auditStatus: issue.displayStatus === "done" ? "passed" : issue.displayStatus === "review" ? "failed" : "not-requested",
    })),
  };
}

export function createBrowserPreviewExecuteStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): ExecuteStatusSnapshot {
  return {
    version: "execute-status.browser-preview",
    projectRoot,
    status: "ready",
    ready: true,
    manifestExists: true,
    indexExists: true,
    summary: {
      runs: 0,
      activeRuns: 0,
      blockedRuns: 0,
      completedRuns: 0,
      activeLeases: 0,
    },
    missingPaths: [],
    warnings: ["浏览器预览只展示 mock execute 状态，不执行命令、不应用 patch。"],
    errors: [],
  };
}

export function createBrowserPreviewOutputStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): OutputStatusSnapshot {
  return {
    version: "output-status.browser-preview",
    projectRoot,
    status: "ready",
    ready: true,
    manifestExists: true,
    indexExists: true,
    summary: {
      evidence: 1,
      releaseDeliveries: 1,
      audits: 1,
      logs: 0,
      backups: 0,
      incompleteEvidence: 0,
      incompleteDeliveries: 0,
    },
    missingPaths: [],
    warnings: ["浏览器预览只展示 mock output 状态；human audit 触发只在真实客户端 Tauri 命令中写 output/audit。"],
    errors: [],
  };
}

export function createBrowserPreviewOutputIndex(): OutputIndex {
  const releaseDelivery = browserPreviewOutputEntry(
    previewDeliveryRunId,
    previewIssueId,
    previewSpecId,
    ".agentflow/output/release/run-browser-preview-001/delivery.json",
    "delivered",
  );
  return {
    version: "output-index.browser-preview",
    updatedAt: previewTimestamp,
    evidence: [
      browserPreviewOutputEntry(
        previewDeliveryRunId,
        previewIssueId,
        previewSpecId,
        ".agentflow/output/evidence/run-browser-preview-001.json",
        "complete",
      ),
    ],
    releaseDeliveries: [releaseDelivery],
    audits: [
      browserPreviewOutputEntry(
        previewDeliveryRunId,
        previewIssueId,
        previewSpecId,
        ".agentflow/output/audit/audit-browser-preview-001/audit-report.md",
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
        trigger: "release-auto",
        requestedBy: "agentflow-release-auto",
        requestedAt: previewTimestamp,
        sourceDeliveryId: previewDeliveryRunId,
        sourceRunId: previewDeliveryRunId,
        sourceIssueId: previewIssueId,
        sourceSpecId: previewSpecId,
        reportPath: ".agentflow/output/audit/audit-browser-preview-001/audit-report.md",
        auditPath: ".agentflow/output/audit/audit-browser-preview-001/audit.json",
      },
    ],
  };
}

export function createBrowserPreviewHumanAuditReport(): HumanAuditReport | null {
  return {
    request: {
      trigger: "release-auto",
      reason: "审计请求已独立登记，用于核对交付材料。",
      source: {
        kind: "release-delivery",
        deliveryId: previewDeliveryRunId,
        runId: previewDeliveryRunId,
        issueId: previewIssueId,
        specId: previewSpecId,
      },
      scope: {
        description: "交付关联审计 Build Agent delivery。",
        refs: [
          {
            kind: "spec",
            id: previewSpecId,
            path: `.agentflow/input/specs/approved/${previewSpecId}/`,
          },
          {
            kind: "issue",
            id: previewIssueId,
            path: `.agentflow/input/issues/${previewIssueId}.json`,
          },
          {
            kind: "execute-run",
            id: previewDeliveryRunId,
            path: `.agentflow/execute/runs/${previewDeliveryRunId}/`,
          },
          {
            kind: "evidence",
            id: previewDeliveryRunId,
            path: `.agentflow/output/evidence/${previewDeliveryRunId}.json`,
          },
          {
            kind: "release-delivery",
            id: previewDeliveryRunId,
            path: `.agentflow/output/release/${previewDeliveryRunId}/delivery.json`,
          },
        ],
      },
    },
    audit: {
      auditId: previewAuditId,
      status: "passed-with-warnings",
      trigger: "release-auto",
      requestedBy: "agentflow-release-auto",
      requestedAt: previewTimestamp,
      sourceDeliveryId: previewDeliveryRunId,
      sourceRunId: previewDeliveryRunId,
      sourceIssueId: previewIssueId,
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
        report: ".agentflow/output/audit/audit-browser-preview-001/audit-report.md",
      },
    },
    reportMarkdown:
      "# Human Audit Browser Preview\n\n" +
      "状态：通过，有警告。\n\n" +
      "- 已核对 release delivery、execute run、evidence 和 issue scope refs。\n" +
      "- 浏览器预览只展示 mock 审计报告，不写 `.agentflow/output/audit`。\n",
    findings: [
      {
        id: "finding-browser-preview-001",
        severity: "warning",
        summary: "Browser Preview 使用 mock audit package。",
      },
    ],
    checklistMarkdown:
      "- [x] Scope refs 自动生成\n" +
      "- [x] Request Human Audit 在浏览器预览中禁用\n" +
      "- [x] audit-report.md 可只读展示\n",
    evidenceMap: {
      evidence: [".agentflow/output/evidence/run-browser-preview-001.json"],
      releaseDelivery: [".agentflow/output/release/run-browser-preview-001/delivery.json"],
    },
    traceability: {
      sourceSpecId: previewSpecId,
      issueId: previewIssueId,
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
    browserPreviewDirectoryEntry("crates", [browserPreviewDirectoryChild("crates/agentflow-core")]),
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
  "crates/agentflow-core",
  "crates/agentflow-core/src",
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
    crates: [{ relativePath: "crates/agentflow-core", kind: "directory" }],
    "crates/agentflow-core": [{ relativePath: "crates/agentflow-core/src", kind: "directory" }],
    "crates/agentflow-core/src": [{ relativePath: "crates/agentflow-core/src/lib.rs", kind: "file" }],
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
      content: '[workspace]\nmembers = ["crates/agentflow-core", "apps/desktop/src-tauri"]\nresolver = "2"\n',
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
    "crates/agentflow-core/src/lib.rs": {
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
