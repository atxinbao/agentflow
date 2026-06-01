import type {
  GraphContextPack,
  GraphManifestSnapshot,
  GraphSearchSnapshot,
  GraphStatusSnapshot,
  IssueContract,
  LocalMetricsSnapshot,
  LocalProjectModelSnapshot,
  LocalSearchSnapshot,
  ProjectDirectoryPage,
  ProjectFileChild,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileSearchSnapshot,
  ProjectFileViewMode,
  ProjectFilesSnapshot,
  ProjectMilestoneIssueViewModelSnapshot,
  WorkbenchBoundary,
  WorkbenchSnapshot,
} from "./types";
import { getProjectFileExtensionFromName, normalizeProjectRelativePath } from "./features/project-files/projectFileUtils";

export const BROWSER_PREVIEW_PROJECT_ROOT = "/Users/mac/Documents/AgentFlow";

const previewBoundary: WorkbenchBoundary = {
  readOnly: true,
  disallowedActions: ["不执行命令", "不写入项目文件", "不调用模型", "不创建远程对象"],
};

const previewTimestamp = 1780291200;

const previewIssueContract: IssueContract = {
  id: "ISSUE-PREVIEW-001",
  title: "浏览器预览文件阅读器",
  status: "todo",
  intent: "验证浏览器预览环境下的项目文件阅读器、文件列表和只读边界。",
  scope: ["展示浏览器预览专用文件树。", "展示 Markdown、配置文件、代码和目录概览。", "保持真实桌面客户端只读取真实本地文件。"],
  nonGoals: ["不执行命令。", "不写入本地工作区。", "不调用模型。", "不创建远程对象。"],
  context: {
    repo: BROWSER_PREVIEW_PROJECT_ROOT,
    files: ["apps/desktop/src/App.tsx", "apps/desktop/src/features/project-files/useProjectFiles.ts"],
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
        forbiddenFiles: [".agentflow/*", ".codex/*", "graphify-out/*"],
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
    searchedPaths: ["README.md", "apps/desktop/src/App.tsx", "apps/desktop/src/features/project-files/useProjectFiles.ts"],
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

export function createBrowserPreviewGraphStatus(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): GraphStatusSnapshot {
  return {
    version: "graph-status.browser-preview",
    projectRoot,
    status: "ready",
    fileCount: 9,
    symbolCount: 18,
    relationCount: 12,
    updatedAt: previewTimestamp,
    lastError: null,
    watcherStatus: "mock",
    preflightStatus: "ready",
    protectionStatus: "ready",
    degradedReasons: [],
  };
}

export function createBrowserPreviewGraphManifest(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): GraphManifestSnapshot {
  return {
    version: "graph-manifest.browser-preview",
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

export function createBrowserPreviewGraphSearch(query: string): GraphSearchSnapshot {
  const normalizedQuery = query.trim() || "project";
  return {
    version: "graph-search.browser-preview",
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
        path: "crates/graph/src/manager.rs",
        title: "prepare_project_graph",
        language: "rust",
        symbolKind: "function",
        line: 28,
        snippet: "准备本地 Graph V1 索引目录和状态。",
        score: 0.82,
      },
    ],
  };
}

export function createBrowserPreviewGraphContextPack(projectRoot = BROWSER_PREVIEW_PROJECT_ROOT): GraphContextPack {
  return {
    version: "graph-context-pack.browser-preview",
    targetType: "preview",
    targetId: "browser-preview",
    query: "Project 文件阅读器 Graph V1 浏览器预览",
    createdAt: previewTimestamp,
    graphRevision: "browser-preview",
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
        path: "apps/desktop/src/features/project-files/useProjectFiles.ts",
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
  const childrenByPath: Record<string, ProjectFileChild[]> = {
    ".git": [browserPreviewFileChild(".git/HEAD"), browserPreviewFileChild(".git/config")],
    apps: [browserPreviewDirectoryChild("apps/desktop")],
    "apps/desktop": [browserPreviewFileChild("apps/desktop/package.json"), browserPreviewDirectoryChild("apps/desktop/src")],
    "apps/desktop/src": [browserPreviewFileChild("apps/desktop/src/App.tsx"), browserPreviewDirectoryChild("apps/desktop/src/features")],
    "apps/desktop/src/features": [browserPreviewDirectoryChild("apps/desktop/src/features/project-files")],
    "apps/desktop/src/features/project-files": [
      browserPreviewFileChild("apps/desktop/src/features/project-files/useProjectFiles.ts"),
      browserPreviewFileChild("apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx"),
    ],
    crates: [browserPreviewDirectoryChild("crates/agentflow-core")],
    "crates/agentflow-core": [browserPreviewDirectoryChild("crates/agentflow-core/src")],
    "crates/agentflow-core/src": [browserPreviewFileChild("crates/agentflow-core/src/lib.rs")],
    docs: [browserPreviewDirectoryChild("docs/requirements")],
    "docs/requirements": [browserPreviewFileChild("docs/requirements/001-add-local-project.md")],
    target: [browserPreviewDirectoryChild("target/debug")],
    "target/debug": [],
  };
  return childrenByPath[path] ?? [];
}

function findBrowserPreviewEntry(relativePath: string): ProjectFileEntry | null {
  const normalizedPath = normalizeProjectRelativePath(relativePath);
  const topLevelEntries = browserPreviewTopLevelEntries();
  const directTopLevel = topLevelEntries.find((entry) => entry.relativePath === normalizedPath);
  if (directTopLevel) {
    return directTopLevel;
  }
  const name = normalizedPath.split("/").at(-1) ?? normalizedPath;
  const isDirectory = browserPreviewDirectoryChildren(normalizedPath).length > 0 || ["apps/desktop", "apps/desktop/src", "crates/agentflow-core", "docs/requirements", "target/debug"].includes(normalizedPath);
  return {
    name,
    relativePath: normalizedPath,
    kind: isDirectory ? "directory" : "file",
    createdAt: previewTimestamp,
    modifiedAt: previewTimestamp,
    sizeBytes: isDirectory ? null : browserPreviewFileContentByPath(normalizedPath, BROWSER_PREVIEW_PROJECT_ROOT).content.length,
    extension: isDirectory ? null : getProjectFileExtensionFromName(name),
    childCount: isDirectory ? browserPreviewDirectoryChildren(normalizedPath).length : null,
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
    childCount: browserPreviewDirectoryChildren(relativePath).length,
    isSymlink: false,
  };
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
    .some((part) => [".git", ".agentflow", ".codex", "target", "node_modules", "dist", "build", "graphify-out"].includes(part));
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
      content: `# AgentFlow\n\n浏览器预览模式使用这份 mock 项目数据来验证 Desktop UI。\n\n## 边界\n\n- 真实桌面客户端读取 ${projectRoot} 下的本地文件。\n- 浏览器预览不具备 Tauri 本地命令能力，因此只展示 mock 文件树。\n- 浏览器预览不会写入 .agentflow/、.codex/ 或 graphify-out/。\n`,
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
    "apps/desktop/src/features/project-files/useProjectFiles.ts": {
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
