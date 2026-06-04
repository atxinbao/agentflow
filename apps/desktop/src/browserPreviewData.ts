import type {
  PanelContextPack,
  PanelManifestSnapshot,
  PanelSearchSnapshot,
  PanelStatusSnapshot,
  GoalTreeSnapshot,
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
  InputStatusSnapshot,
  ExecuteStatusSnapshot,
  OutputStatusSnapshot,
  OutputIndex,
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
const previewGoalId = "goal-001";
const previewMilestoneId = "ms-001";
const previewIssueId = "iss-001";

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
        status: "todo",
        rawStatus: "todo",
        goal: previewIssueContract.intent,
        scope: previewIssueContract.scope,
        nonGoals: previewIssueContract.nonGoals,
        dependencies: [],
        codexInstructions: previewIssueContract.executionPlan,
        acceptanceCriteria: ["浏览器预览可展示 mock 文件内容。", "真实客户端不使用 mock fallback。"],
        validationCommands: previewIssueContract.validation.commands,
        evidenceRequired: previewIssueContract.evidenceRequirements,
        allowedFiles: previewIssueContract.context.files,
        forbiddenFiles: [".agentflow/*", ".codex/*", "agent-artifacts/*"],
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

export function createBrowserPreviewGoalTreeSnapshot(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): GoalTreeSnapshot {
  return {
    version: "goal-tree-snapshot.browser-preview",
    projectRoot,
    index: {
      version: "goal-tree.v1",
      projectRoot,
      activeGoalId: previewGoalId,
      goalOrder: [previewGoalId],
      milestoneOrderByGoal: {
        [previewGoalId]: [previewMilestoneId],
      },
      issueOrderByMilestone: {
        [previewMilestoneId]: [previewIssueId],
      },
      updatedAt: previewTimestamp,
    },
    goals: [
      {
        version: "goal.v1",
        id: previewGoalId,
        projectRoot,
        status: "active",
        human: {
          title: "Goal Tree Agent-only 边界",
          objective: "把 Goal Tree 定义为 Agent 使用的目标工作地图，Desktop 人类界面只读查看。",
          scope: ["保留 .agentflow/define/** 事实源。", "Desktop 只读取目标树、完整性提示和 Agent Draft。", "写入能力收敛到未来 Agent / System 通道。"],
          nonGoals: ["不启动 Agent。", "不执行项目命令。", "不调用模型。", "不创建远程对象。"],
          successCriteria: ["目标树可读取。", "Desktop 不显示写入入口。", "完整性 warning 可见。"],
          milestoneOrder: [previewMilestoneId],
          validationGate: ["cargo test -p agentflow-goal-tree", "npm --prefix apps/desktop run build"],
          closureGate: ["人类 UI 不写 .agentflow/define/**。", "人类 UI 不写 Panel Context Pack。"],
        },
        agentDraft: {
          suggestedMilestones: ["Desktop 只读收敛", "Tauri command 收窄"],
          suggestedRisks: ["Panel 缺失时推荐上下文不完整。"],
          suggestedQuestions: ["未来 Agent planning flow 如何授权写入？"],
          suggestedIssueBreakdown: ["移除 Desktop 写入入口", "收窄 Tauri command", "标注 agent-only API"],
        },
        system: {
          createdAt: previewTimestamp,
          updatedAt: previewTimestamp,
          createdBy: "agent-system",
          updatedBy: "agent-system",
          path: `.agentflow/define/goals/${previewGoalId}.json`,
          revision: 1,
        },
      },
    ],
    milestones: [
      {
        version: "milestone.v1",
        id: previewMilestoneId,
        goalId: previewGoalId,
        projectRoot,
        status: "active",
        human: {
          title: "只读界面边界",
          stageGoal: "把 Desktop Goal Tree 页面收敛为只读查看界面。",
          entryCriteria: ["项目已接入 AgentFlow。", "Project File Reader 可读取本地文件。"],
          scope: ["显示 Confirmed Contract。", "显示 Agent Draft。", "显示 System State。", "显示完整性提示。"],
          nonGoals: ["不实现 AgentRun。", "不实现 Lease。"],
          issueOrder: [previewIssueId],
          exitCriteria: ["页面无写入按钮。", "完整性校验可读。", "浏览器预览只读。"],
          nextGate: ["未来 Agent planning flow 定义授权写入通道。"],
        },
        agentDraft: {
          suggestedIssues: ["Desktop 只读 UI", "Tauri read-only registry"],
          suggestedRisks: ["旧 workflow 类型误入新主线。"],
          suggestedQuestions: [],
        },
        system: {
          createdAt: previewTimestamp,
          updatedAt: previewTimestamp,
          createdBy: "agent-system",
          updatedBy: "agent-system",
          path: `.agentflow/define/milestones/${previewMilestoneId}.json`,
          revision: 1,
        },
      },
    ],
    issues: [
      {
        version: "issue.v1",
        id: previewIssueId,
        goalId: previewGoalId,
        milestoneId: previewMilestoneId,
        projectRoot,
        status: "ready",
        human: {
          title: "移除人类写入入口",
          goal: "让 Desktop 只能查看 Goal / Milestone / Issue 目标约束和系统状态。",
          scope: ["移除 Goal Tree 页面写入按钮。", "Tauri handler 只保留读取和验证命令。"],
          nonGoals: ["不复用旧 IssueContract。", "不写 .agentflow/runs 或 evidence。"],
          dependencies: [],
          acceptanceCriteria: ["页面不显示写入按钮。", "validate 能输出 warning。", "Browser Preview 不触发写入。"],
          validationCommands: ["cargo test -p agentflow-goal-tree"],
          evidenceRequirements: ["测试输出。", "只读边界说明。"],
          boundary: ["不启动 Agent。", "不执行用户项目命令。", "不调用模型。"],
        },
        agentDraft: {
          suggestedFiles: ["crates/goal-tree/src/lib.rs", "crates/goal-tree/src/manager.rs"],
          suggestedSymbols: [],
          suggestedTests: ["creates_goal_tree_records_under_define_paths"],
          suggestedImplementationPlan: ["定义模型", "实现 atomic write", "实现 validation"],
          suggestedRisks: ["旧路径误写。"],
          questions: [],
        },
        system: {
          createdAt: previewTimestamp,
          updatedAt: previewTimestamp,
          createdBy: "agent-system",
          updatedBy: "agent-system",
          path: `.agentflow/define/issues/${previewIssueId}.json`,
          revision: 1,
          panelContextPackPath: ".agentflow/panel/context-packs/iss-001.json",
        },
      },
    ],
    validation: {
      version: "goal-tree-validation.v1",
      projectRoot,
      valid: true,
      errors: [],
      warnings: [
        {
          code: "browser_preview_mock",
          message: "浏览器预览使用 mock Goal Tree，不写真实 .agentflow/。",
          objectType: "goal-tree",
          objectId: null,
        },
      ],
    },
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
      skillCount: 6,
    },
    skills: [
      "request-triage",
      "requirement-intake-filter",
      "spec-gate-authoring",
      "input-issue-generation",
      "boundary-check",
      "validation",
    ].map((name) => ({
      name,
      path: `.agentflow/define/agent/skills/${name}/SKILL.md`,
      exists: true,
      hashMatches: true,
      version: "v1",
    })),
    repairs: [],
    warnings: ["浏览器预览只展示 mock Agent Manual 状态，不写 AGENTS.md。"],
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
      approvedSpecs: 0,
      projects: 0,
      issues: 0,
      blockedIssues: 0,
      highRiskIssues: 0,
    },
    missingPaths: [],
    warnings: ["浏览器预览只展示 mock input 状态，不写 .agentflow/input。"],
    errors: [],
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
      evidence: 0,
      releaseDeliveries: 0,
      audits: 0,
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
  return {
    version: "output-index.browser-preview",
    updatedAt: previewTimestamp,
    evidence: [],
    releaseDeliveries: [],
    audits: [],
  };
}

export function createBrowserPreviewAuditIndex(): AuditIndex {
  return {
    version: "audit-index.browser-preview",
    updatedAt: previewTimestamp,
    audits: [],
  };
}

export function createBrowserPreviewHumanAuditReport(): HumanAuditReport | null {
  return null;
}

export function createBrowserPreviewStateStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): StateStatusSnapshot {
  return {
    version: "state-status.browser-preview",
    projectRoot,
    status: "ready",
    currentStage: "workspace-ready",
    auditStatus: "not-requested",
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
