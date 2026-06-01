import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  BarChart3,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  ClipboardList,
  FileSearch,
  FileText,
  Flag,
  FolderKanban,
  FolderPlus,
  GitBranch,
  History,
  LayoutDashboard,
  ListChecks,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  UsersRound,
  X,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useState, type CSSProperties, type FormEvent } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewMetricsSnapshot,
  createBrowserPreviewProjectModelSnapshot,
  createBrowserPreviewProjectViewModelSnapshot,
  createBrowserPreviewSearchSnapshot,
  createBrowserPreviewWorkbenchSnapshot,
} from "./browserPreviewData";
import {
  ProjectLocalFilesPage,
  isBrowserPreviewRuntime,
  normalizeProjectRootKey,
  projectNameFromPath,
  projectRootsEqual,
  useProjectGraph,
  useProjectFiles,
  type ProjectGraphState,
  type ProjectFilesState,
} from "./features/project-files";
import type {
  AgentRun,
  GoalLoopSelection,
  GoalLoopState,
  IssueContract,
  LocalMetricsSnapshot,
  MilestoneDerivedProgress,
  LocalProjectModelSnapshot,
  LocalSearchSnapshot,
  ProjectMilestoneIssueViewModelSnapshot,
  V1Issue,
  V1Milestone,
  V1Project,
  V1View,
  V1ViewSort,
  WorkbenchSnapshot,
  WorkbenchTextArtifact,
  WorkbenchBoundary,
} from "./types";

type ViewKey =
  | "overview"
  | "teams"
  | "goal-loop"
  | "lifecycle"
  | "timeline"
  | "projects"
  | "issues"
  | "metrics"
  | "search"
  | "evidence"
  | "reviews"
  | "views";

type LoadState = {
  snapshot: WorkbenchSnapshot | null;
  metrics: LocalMetricsSnapshot | null;
  projectModel: LocalProjectModelSnapshot | null;
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null;
  error: string | null;
  source: "tauri" | "preview" | "unavailable" | "loading";
};

type SearchState = {
  snapshot: LocalSearchSnapshot | null;
  error: string | null;
  loading: boolean;
  source: "idle" | "tauri" | "preview" | "unavailable";
};

type LocalProjectFolder = {
  id: string;
  name: string;
  root: string;
  agentflowPath?: string | null;
  gitProtected?: boolean;
  preparedAt?: string | null;
};

const LOCAL_PROJECT_FOLDERS_STORAGE_KEY = "agentflow.localProjectFolders.v1";

const emptyBoundary: WorkbenchBoundary = {
  readOnly: true,
  disallowedActions: ["不执行命令", "不写入项目文件", "不调用模型", "不创建远程对象"],
};

type ProjectWorkspaceSummary = {
  version: string;
  id: string;
  name: string;
  root: string;
  agentflowPath: string;
  workspacePath: string;
  configPath: string;
  createdAgentflow: boolean;
  createdPaths: string[];
  reusedPaths: string[];
  gitExcludePath?: string | null;
  protectedGitExclude: boolean;
};

type SidebarProjectItem = {
  id: string;
  name: string;
  root: string | null;
  modelProject: V1Project | null;
};

type TaskProgressSnapshot = {
  total: number;
  done: number;
  active: number;
  pending: number;
  canceled: number;
  donePercent: number;
  activePercent: number;
  pendingPercent: number;
  canceledPercent: number;
};

function readStoredLocalProjectFolders(): LocalProjectFolder[] {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const rawValue = window.localStorage.getItem(LOCAL_PROJECT_FOLDERS_STORAGE_KEY);
    return rawValue ? cleanStoredLocalProjectFolders(JSON.parse(rawValue)) : [];
  } catch {
    return [];
  }
}

function storeLocalProjectFolders(projects: LocalProjectFolder[]) {
  if (typeof window === "undefined") {
    return;
  }

  const normalizedProjects = cleanStoredLocalProjectFolders(projects);
  try {
    window.localStorage.setItem(LOCAL_PROJECT_FOLDERS_STORAGE_KEY, JSON.stringify(normalizedProjects));
  } catch {
    // 本地 UI 状态持久化失败不影响文件阅读器主流程。
  }
}

function cleanStoredLocalProjectFolders(value: unknown): LocalProjectFolder[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.reduce<LocalProjectFolder[]>((projects, item) => {
    if (!item || typeof item !== "object" || !("root" in item) || typeof item.root !== "string") {
      return projects;
    }
    const projectRoot = normalizeProjectRootKey(item.root);
    if (!projectRoot) {
      return projects;
    }
    if (isLegacyMockProjectFolder(item, projectRoot)) {
      return projects;
    }
    return upsertLocalProjectFolder(projects, localProjectFolderFromRoot(projectRoot, item));
  }, []);
}

function isLegacyMockProjectFolder(item: Record<string, unknown>, projectRoot: string) {
  const name = typeof item.name === "string" ? item.name : "";
  return name === "AgentFlow-Preview-Project" || projectRoot.endsWith("/AgentFlow-Preview-Project");
}

function createEmptyWorkbenchSnapshot(projectRoot = ""): WorkbenchSnapshot {
  return {
    version: "workbench.empty",
    initialized: false,
    projectRoot,
    projectSummaryMarkdown: null,
    goalLoopSummaryMarkdown: null,
    goalLoop: null,
    issues: [],
    runs: [],
    savedViews: [],
    evidence: [],
    reviews: [],
    projectUpdates: [],
    counts: {
      issues: 0,
      completedIssues: 0,
      runs: 0,
      passedRuns: 0,
      evidenceReports: 0,
      reviews: 0,
      projectUpdates: 0,
      savedViews: 0,
    },
    boundary: emptyBoundary,
  };
}

function createEmptyMetricsSnapshot(projectRoot = ""): LocalMetricsSnapshot {
  return {
    version: "metrics.empty",
    initialized: false,
    projectRoot,
    issues: {
      total: 0,
      completed: 0,
      planned: 0,
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
      savedViews: 0,
    },
    goalReady: false,
    activeIssueId: null,
    nextAction: "添加项目",
    recommendedCommand: "agentflow project create",
    sources: [],
    boundary: emptyBoundary,
  };
}

function createEmptyProjectModelSnapshot(projectRoot = ""): LocalProjectModelSnapshot {
  return {
    version: "project-model.empty",
    initialized: false,
    projectRoot,
    workspace: null,
    teams: [],
    projects: [],
    issueRefs: [],
    goalLoopSelection: {
      activeProjectId: null,
      source: "empty",
      nextAction: "添加项目",
      nextIssueIntent: null,
      recommendedCommand: "agentflow project create",
      rationale: ["当前没有从本地事实源读取到 Project。"],
    },
    sources: [],
    boundary: emptyBoundary,
  };
}

function createEmptyProjectViewModelSnapshot(projectRoot = ""): ProjectMilestoneIssueViewModelSnapshot {
  return {
    version: "project-view-model.empty",
    initialized: false,
    projectRoot,
    workspace: null,
    teams: [],
    projects: [],
    issues: [],
    views: [],
    invariants: ["仅显示真实本地项目数据；未读取到事实源时不注入示例项目。"],
    sources: [],
    boundary: emptyBoundary,
  };
}

function App() {
  const [loadState, setLoadState] = useState<LoadState>({
    snapshot: null,
    metrics: null,
    projectModel: null,
    projectViewModel: null,
    error: null,
    source: "loading",
  });
  const [activeView, setActiveView] = useState<ViewKey>("projects");
  const [teamCreateOpen, setTeamCreateOpen] = useState(false);
  const [selectedTeamId, setSelectedTeamId] = useState<string | null>(null);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [selectedProjectRoot, setSelectedProjectRoot] = useState<string | null>(null);
  const [localProjectFolders, setLocalProjectFolders] = useState<LocalProjectFolder[]>(readStoredLocalProjectFolders);
  const [projectAddOpen, setProjectAddOpen] = useState(false);
  const [projectAddPath, setProjectAddPath] = useState("");
  const [projectAddFeedback, setProjectAddFeedback] = useState<string | null>(null);
  const [expandedProjectIds, setExpandedProjectIds] = useState<Set<string>>(() => new Set());
  const [selectedMilestoneId, setSelectedMilestoneId] = useState<string | null>(null);
  const [selectedIssueId, setSelectedIssueId] = useState<string | null>(null);
  const [selectedViewId, setSelectedViewId] = useState<string | null>(null);
  const [selectedArtifactPath, setSelectedArtifactPath] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("桌面搜索");
  const [searchState, setSearchState] = useState<SearchState>({
    snapshot: null,
    error: null,
    loading: false,
    source: "idle",
  });
  const {
    clearProjectFilesError,
    loadProjectFiles,
    projectFilesState,
    reportProjectFilesError,
    selectProjectFile,
  } = useProjectFiles(selectedProjectRoot);
  const graphProjectRoot =
    selectedProjectRoot ??
    projectFilesState.snapshot?.projectRoot ??
    (isBrowserPreviewRuntime() ? BROWSER_PREVIEW_PROJECT_ROOT : null);
  const { projectGraphState } = useProjectGraph(graphProjectRoot);

  async function loadSnapshot() {
    setLoadState((current) => ({ ...current, error: null }));
    try {
      const [snapshot, metrics, projectModel, projectViewModel] = await Promise.all([
        invoke<WorkbenchSnapshot>("load_workbench_snapshot"),
        invoke<LocalMetricsSnapshot>("load_metrics_snapshot"),
        invoke<LocalProjectModelSnapshot>("load_project_model_snapshot"),
        invoke<ProjectMilestoneIssueViewModelSnapshot>("load_project_milestone_issue_view_model_snapshot"),
      ]);
      setLoadState({ snapshot, metrics, projectModel, projectViewModel, error: null, source: "tauri" });
      setSelectedTeamId((current) => current ?? projectViewModel.teams.at(0)?.id ?? null);
      const defaultProjectId =
        projectViewModel.workspace?.activeProjectId ??
        [...projectViewModel.projects].sort(compareProjects).at(0)?.id ??
        null;
      setSelectedProjectId((current) => current ?? defaultProjectId);
      if (defaultProjectId) {
        setExpandedProjectIds((current) => (current.size > 0 ? current : new Set([defaultProjectId])));
      }
      setSelectedViewId((current) => current ?? [...projectViewModel.views].sort(compareViews).at(0)?.id ?? null);
      setSelectedIssueId((current) => current ?? sortedV1Issues(projectViewModel.issues).at(0)?.id ?? snapshot.issues.at(-1)?.id ?? null);
      setSelectedArtifactPath(
        (current) => current ?? snapshot.evidence.at(-1)?.path ?? snapshot.reviews.at(-1)?.path ?? null,
      );
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      const projectRoot = selectedProjectRoot ?? projectFilesState.snapshot?.projectRoot ?? (isBrowserPreviewRuntime() ? BROWSER_PREVIEW_PROJECT_ROOT : "");
      if (isBrowserPreviewRuntime()) {
        const previewSnapshot = createBrowserPreviewWorkbenchSnapshot(projectRoot);
        const previewMetrics = createBrowserPreviewMetricsSnapshot(projectRoot);
        const previewProjectModel = createBrowserPreviewProjectModelSnapshot(projectRoot);
        const previewProjectViewModel = createBrowserPreviewProjectViewModelSnapshot(projectRoot);
        setLoadState({
          snapshot: previewSnapshot,
          metrics: previewMetrics,
          projectModel: previewProjectModel,
          projectViewModel: previewProjectViewModel,
          error: null,
          source: "preview",
        });
        const defaultProjectId = previewProjectViewModel.workspace?.activeProjectId ?? previewProjectViewModel.projects.at(0)?.id ?? null;
        setSelectedTeamId((current) => current ?? previewProjectViewModel.teams.at(0)?.id ?? null);
        setSelectedProjectId((current) => current ?? defaultProjectId);
        setSelectedProjectRoot((current) => current ?? projectRoot);
        if (defaultProjectId) {
          setExpandedProjectIds((current) => (current.size > 0 ? current : new Set([defaultProjectId])));
        }
        setSelectedViewId((current) => current ?? previewProjectViewModel.views.at(0)?.id ?? null);
        setSelectedIssueId((current) => current ?? sortedV1Issues(previewProjectViewModel.issues).at(0)?.id ?? null);
        setSelectedArtifactPath((current) => current ?? null);
        return;
      }
      setLoadState({
        snapshot: createEmptyWorkbenchSnapshot(projectRoot),
        metrics: createEmptyMetricsSnapshot(projectRoot),
        projectModel: createEmptyProjectModelSnapshot(projectRoot),
        projectViewModel: createEmptyProjectViewModelSnapshot(projectRoot),
        error: errorMessage,
        source: "unavailable",
      });
      setSelectedTeamId((current) => current ?? null);
      setSelectedProjectId((current) => current ?? localProjectFolders.at(0)?.id ?? null);
      setSelectedViewId((current) => current ?? null);
      setSelectedIssueId((current) => current ?? null);
      setSelectedArtifactPath((current) => current ?? null);
    }
  }

  async function loadSearchSnapshot(query: string) {
    const trimmedQuery = query.trim();
    if (!trimmedQuery) {
      setSearchState({ snapshot: null, error: null, loading: false, source: "idle" });
      return;
    }

    setSearchState((current) => ({ ...current, error: null, loading: true }));
    try {
      const snapshot = await invoke<LocalSearchSnapshot>("load_search_snapshot", { query: trimmedQuery });
      setSearchState({ snapshot, error: null, loading: false, source: "tauri" });
    } catch (error) {
      if (isBrowserPreviewRuntime()) {
        setSearchState({
          snapshot: createBrowserPreviewSearchSnapshot(trimmedQuery, selectedProjectRoot ?? projectFilesState.snapshot?.projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
          error: null,
          loading: false,
          source: "preview",
        });
        return;
      }
      setSearchState({
        snapshot: null,
        error: error instanceof Error ? error.message : String(error),
        loading: false,
        source: "unavailable",
      });
    }
  }

  async function addProjectRoot(projectRootInput: string) {
    let projectRoot = normalizeProjectRootKey(projectRootInput);
    if (!projectRoot) {
      setProjectAddFeedback("请输入项目本地路径。");
      return;
    }

    clearProjectFilesError();
    let workspaceSummary: ProjectWorkspaceSummary | null = null;
    if (!isBrowserPreviewRuntime()) {
      try {
        workspaceSummary = await invoke<ProjectWorkspaceSummary>("prepare_local_project_workspace", { projectRoot });
        projectRoot = normalizeProjectRootKey(workspaceSummary.root);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        setProjectAddFeedback(errorMessage);
        reportProjectFilesError(errorMessage);
        return;
      }
    }

    const existingModelProject = findModelProjectForRoot(
      loadState.projectViewModel?.projects ?? [],
      projectRoot,
      loadState.snapshot?.projectRoot ?? null,
    );
    if (existingModelProject) {
      setLocalProjectFolders((current) => removeLocalProjectFolderByRoot(current, projectRoot));
      setExpandedProjectIds((current) => new Set([...current, existingModelProject.id]));
      setSelectedProjectRoot(projectRoot);
      setSelectedProjectId(existingModelProject.id);
      setActiveView("projects");
      setSelectedMilestoneId(null);
      setProjectAddOpen(false);
      setProjectAddFeedback(workspaceSummary ? projectWorkspaceFeedback(workspaceSummary, "项目已存在，已准备并切换。") : "项目已存在，已切换。");
      await loadProjectFiles(projectRoot);
      return;
    }

    const existingLocalProject = localProjectFolders.find((project) => projectRootsEqual(project.root, projectRoot));
    if (existingLocalProject) {
      setExpandedProjectIds((current) => new Set([...current, existingLocalProject.id]));
      setSelectedProjectRoot(existingLocalProject.root);
      setSelectedProjectId(existingLocalProject.id);
      setActiveView("projects");
      setSelectedMilestoneId(null);
      setProjectAddOpen(false);
      setProjectAddFeedback(workspaceSummary ? projectWorkspaceFeedback(workspaceSummary, "项目已存在，已准备并切换。") : "项目已存在，已切换。");
      await loadProjectFiles(existingLocalProject.root);
      return;
    }

    const localProject = localProjectFolderFromRoot(projectRoot, workspaceSummary);
    setLocalProjectFolders((current) => upsertLocalProjectFolder(current, localProject));
    setExpandedProjectIds((current) => new Set([...current, localProject.id]));
    setSelectedProjectRoot(projectRoot);
    setSelectedProjectId(localProject.id);
    setActiveView("projects");
    setSelectedMilestoneId(null);
    setProjectAddOpen(false);
    setProjectAddFeedback(workspaceSummary ? projectWorkspaceFeedback(workspaceSummary, `已添加：${localProject.name}`) : `已添加：${localProject.name}`);
    await loadProjectFiles(projectRoot);
  }

  function removeLocalProject(projectId: string, projectRoot: string) {
    const nextLocalProjects = localProjectFolders.filter((project) => project.id !== projectId);
    setLocalProjectFolders(nextLocalProjects);
    setExpandedProjectIds((current) => {
      const next = new Set(current);
      next.delete(projectId);
      return next;
    });
    setProjectAddFeedback("已从列表移除；未删除源码或 .agentflow/。");

    if (selectedProjectId !== projectId) {
      return;
    }

    const fallbackLocalProject = nextLocalProjects.at(0) ?? null;
    const fallbackModelProject = [...(loadState.projectViewModel?.projects ?? [])].sort(compareProjects).at(0) ?? null;
    const fallbackProjectId = fallbackLocalProject?.id ?? fallbackModelProject?.id ?? null;
    const fallbackProjectRoot = fallbackLocalProject?.root ?? (fallbackModelProject ? loadState.snapshot?.projectRoot ?? null : null);
    setSelectedProjectId(fallbackProjectId);
    setSelectedProjectRoot(fallbackProjectRoot);
    setSelectedMilestoneId(null);
    setSelectedIssueId(null);
    setActiveView("projects");
    void loadProjectFiles(fallbackProjectRoot ?? null);
  }

  async function chooseProjectFolder() {
    if (isBrowserPreviewRuntime()) {
      setProjectAddOpen(true);
      setProjectAddPath(
        (current) =>
          current ||
          selectedProjectRoot ||
          projectFilesState.snapshot?.projectRoot ||
          loadState.snapshot?.projectRoot ||
          "/Users/mac/Documents/AgentFlow",
      );
      setProjectAddFeedback("浏览器预览只能保存项目入口；请用桌面客户端读取真实文件。");
      return;
    }

    clearProjectFilesError();
    try {
      const projectRoot = await invoke<string | null>("choose_existing_project_folder");
      if (!projectRoot) {
        return;
      }
      await addProjectRoot(projectRoot);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      setProjectAddFeedback(errorMessage);
      reportProjectFilesError(errorMessage);
    }
  }

  function submitProjectAdd(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void addProjectRoot(projectAddPath);
  }

  useEffect(() => {
    void loadSnapshot();
    void loadProjectFiles();
  }, []);

  useEffect(() => {
    storeLocalProjectFolders(localProjectFolders);
  }, [localProjectFolders]);

  const snapshot = loadState.snapshot;
  const metrics = loadState.metrics;
  const projectModel = loadState.projectModel;
  const projectViewModel = loadState.projectViewModel;
  const issueRuns = useMemo(() => mapRunsByIssue(snapshot?.runs ?? []), [snapshot?.runs]);
  const selectedIssue = useMemo(
    () => snapshot?.issues.find((issue) => issue.id === selectedIssueId) ?? snapshot?.issues.at(-1) ?? null,
    [selectedIssueId, snapshot?.issues],
  );
  const selectedArtifact = useMemo(
    () =>
      [...(snapshot?.evidence ?? []), ...(snapshot?.reviews ?? [])].find(
        (artifact) => artifact.path === selectedArtifactPath,
      ) ?? null,
    [selectedArtifactPath, snapshot?.evidence, snapshot?.reviews],
  );

  if (!snapshot) {
    return (
      <main className="shell loading-shell">
        <RefreshCw className="spin" size={22} />
        <span>正在读取本地 AgentFlow 事实源...</span>
      </main>
    );
  }

  const projectFilesRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot ?? snapshot.projectRoot;
  const projectTopbarTitle = activeView === "projects" ? projectNameFromPath(projectFilesRoot) || "AgentFlow Final v3" : snapshot.initialized ? snapshot.projectRoot : "未找到 .agentflow/ 项目";

  function selectSidebarProject(projectId: string, projectRoot?: string | null) {
    setActiveView("projects");
    setSelectedProjectId(projectId);
    setSelectedMilestoneId(null);
    setSelectedProjectRoot(projectRoot ?? null);
    void loadProjectFiles(projectRoot ?? null);
  }

  function toggleSidebarProject(projectId: string) {
    setExpandedProjectIds((current) => {
      const next = new Set(current);
      if (next.has(projectId)) {
        next.delete(projectId);
      } else {
        next.add(projectId);
      }
      return next;
    });
  }

  return (
    <main className="shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            <svg className="brand-folder-icon" viewBox="0 0 36 28" focusable="false" aria-hidden="true">
              <path d="M3 7.2C3 5.4 4.4 4 6.2 4h9.6c1.1 0 2.1.5 2.7 1.4l1.4 2h10c1.8 0 3.1 1.4 3.1 3.1v11.3c0 1.8-1.4 3.2-3.2 3.2H6.2C4.4 25 3 23.6 3 21.8V7.2Z" />
            </svg>
          </span>
          <div>
            <strong>AgentFlow</strong>
            <span>本地工作台</span>
          </div>
        </div>

        <WorkspaceTreeNav
          activeView={activeView}
          expandedProjectIds={expandedProjectIds}
          localProjectFolders={localProjectFolders}
          onChooseProjectFolder={() => void chooseProjectFolder()}
          onCancelProjectAdd={() => {
            setProjectAddOpen(false);
            setProjectAddFeedback(null);
          }}
          onProjectAddPathChange={setProjectAddPath}
          onSelectIssue={(issueId, projectId, milestoneId, projectRoot) => {
            setActiveView("issues");
            setSelectedIssueId(issueId);
            setSelectedProjectId(projectId ?? null);
            setSelectedMilestoneId(milestoneId ?? null);
            setSelectedProjectRoot(projectRoot ?? null);
          }}
          onSelectMilestone={(projectId, milestoneId, projectRoot) => {
            setActiveView("projects");
            setSelectedProjectId(projectId);
            setSelectedMilestoneId(milestoneId);
            setSelectedProjectRoot(projectRoot ?? null);
            void loadProjectFiles(projectRoot ?? null);
          }}
          onSelectProject={selectSidebarProject}
          onRemoveProject={removeLocalProject}
          onSelectViews={(projectId, projectRoot) => {
            setActiveView("views");
            setSelectedProjectId(projectId);
            setSelectedMilestoneId(null);
            setSelectedProjectRoot(projectRoot ?? null);
          }}
          onToggleProject={toggleSidebarProject}
          projectViewModel={projectViewModel}
          projectAddFeedback={projectAddFeedback}
          projectAddOpen={projectAddOpen}
          projectAddPath={projectAddPath}
          selectedIssueId={selectedIssueId}
          selectedMilestoneId={selectedMilestoneId}
          selectedProjectId={selectedProjectId}
          onSubmitProjectAdd={submitProjectAdd}
          workspaceProjectRoot={snapshot.projectRoot}
        />

        <SidebarFooterLinks />
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            {activeView === "projects" ? null : <p className="eyebrow">本地事实源</p>}
            <h1>{projectTopbarTitle}</h1>
            {activeView === "projects" ? <p className="topbar-path">{projectFilesRoot}</p> : null}
          </div>
          <div className="topbar-actions">
            {activeView === "projects" ? null : (
              <span className={loadState.source === "tauri" ? "source-pill live" : "source-pill"}>
                {loadState.source === "tauri" ? "桌面真实数据" : loadState.source === "preview" ? "浏览器 Mock 数据" : "等待本地数据"}
              </span>
            )}
            <button
              className="icon-command"
              onClick={() => {
                if (activeView === "projects") {
                  void loadProjectFiles(projectFilesRoot);
                  return;
                }
                void loadSnapshot();
                void loadProjectFiles(projectFilesRoot);
              }}
              title={activeView === "projects" ? "重新读取项目文件" : "重新读取本地快照"}
              type="button"
            >
              <RefreshCw size={17} />
            </button>
          </div>
        </header>

        {activeView === "overview" ? <Overview projectModel={projectModel} /> : null}
        {activeView === "teams" ? (
          <TeamView
            createOpen={teamCreateOpen}
            onDismissCreate={() => setTeamCreateOpen(false)}
            onSelectTeam={setSelectedTeamId}
            projectModel={projectModel}
            selectedTeamId={selectedTeamId}
          />
        ) : null}
        {activeView === "goal-loop" ? <GoalLoopTraceView projectModel={projectModel} snapshot={snapshot} /> : null}
        {activeView === "lifecycle" ? (
          <IssueLifecycleTraceView
            evidence={snapshot.evidence}
            issueRuns={issueRuns}
            issues={snapshot.issues}
            onSelectIssue={setSelectedIssueId}
            projectUpdates={snapshot.projectUpdates}
            reviews={snapshot.reviews}
            selectedIssue={selectedIssue}
          />
        ) : null}
        {activeView === "timeline" ? <ProjectUpdateTimelineView snapshot={snapshot} /> : null}
        {activeView === "projects" ? (
          <ProjectView
            onSelectIssue={(issueId) => {
              setActiveView("issues");
              setSelectedIssueId(issueId);
            }}
            onSelectMilestone={setSelectedMilestoneId}
            onSelectProject={setSelectedProjectId}
            onSelectProjectFile={(relativePath) => void selectProjectFile(relativePath)}
            projectGraphState={projectGraphState}
            projectFilesState={projectFilesState}
            projectViewModel={projectViewModel}
            selectedMilestoneId={selectedMilestoneId}
            selectedProjectId={selectedProjectId}
            selectedProjectRoot={selectedProjectRoot}
          />
        ) : null}
        {activeView === "issues" ? (
          <IssuesView
            onSelectIssue={setSelectedIssueId}
            projectViewModel={projectViewModel}
            selectedIssueId={selectedIssueId}
            selectedMilestoneId={selectedMilestoneId}
            selectedProjectId={selectedProjectId}
          />
        ) : null}
        {activeView === "metrics" ? <MetricsView metrics={metrics} /> : null}
        {activeView === "search" ? (
          <SearchView
            goalLoopCommand={snapshot.goalLoop?.recommendedCommand ?? "agentflow goal next"}
            onQueryChange={setSearchQuery}
            onSearch={loadSearchSnapshot}
            query={searchQuery}
            searchState={searchState}
          />
        ) : null}
        {activeView === "evidence" ? (
          <ArtifactView
            artifacts={snapshot.evidence}
            emptyLabel="暂无证据报告。"
            onSelectArtifact={setSelectedArtifactPath}
            selectedArtifact={selectedArtifact}
            title="证据"
          />
        ) : null}
        {activeView === "reviews" ? (
          <ArtifactView
            artifacts={snapshot.reviews}
            emptyLabel="暂无审查报告。"
            onSelectArtifact={setSelectedArtifactPath}
            selectedArtifact={selectedArtifact}
            title="审查"
          />
        ) : null}
        {activeView === "views" ? (
          <SavedViewsView onSelectView={setSelectedViewId} selectedViewId={selectedViewId} views={projectViewModel?.views ?? []} />
        ) : null}
      </section>
    </main>
  );
}

function WorkspaceTreeNav({
  activeView,
  expandedProjectIds,
  localProjectFolders,
  onCancelProjectAdd,
  onChooseProjectFolder,
  onProjectAddPathChange,
  onSelectIssue,
  onSelectMilestone,
  onSelectProject,
  onRemoveProject,
  onSelectViews,
  onSubmitProjectAdd,
  onToggleProject,
  projectAddFeedback,
  projectAddOpen,
  projectAddPath,
  projectViewModel,
  selectedIssueId,
  selectedMilestoneId,
  selectedProjectId,
  workspaceProjectRoot,
}: {
  activeView: ViewKey;
  expandedProjectIds: ReadonlySet<string>;
  localProjectFolders: LocalProjectFolder[];
  onCancelProjectAdd: () => void;
  onChooseProjectFolder: () => void;
  onProjectAddPathChange: (projectPath: string) => void;
  onSelectIssue: (issueId: string, projectId?: string | null, milestoneId?: string | null, projectRoot?: string | null) => void;
  onSelectMilestone: (projectId: string, milestoneId: string, projectRoot?: string | null) => void;
  onSelectProject: (projectId: string, projectRoot?: string | null) => void;
  onRemoveProject: (projectId: string, projectRoot: string) => void;
  onSelectViews: (projectId: string, projectRoot?: string | null) => void;
  onSubmitProjectAdd: (event: FormEvent<HTMLFormElement>) => void;
  onToggleProject: (projectId: string) => void;
  projectAddFeedback: string | null;
  projectAddOpen: boolean;
  projectAddPath: string;
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null;
  selectedIssueId: string | null;
  selectedMilestoneId: string | null;
  selectedProjectId: string | null;
  workspaceProjectRoot: string | null;
}) {
  const workspace = projectViewModel?.workspace ?? null;
  const modelProjects = [...(projectViewModel?.projects ?? [])].sort(compareProjects);
  const projects = buildSidebarProjectItems(modelProjects, localProjectFolders, workspaceProjectRoot);
  const issuesById = new Map((projectViewModel?.issues ?? []).map((issue) => [issue.id, issue]));
  const effectiveProjectId = selectedProjectId ?? workspace?.activeProjectId ?? projects.at(0)?.id ?? null;

  function firstMilestoneForProject(project: V1Project | null) {
    return project ? [...project.milestones].sort(compareMilestones).at(0) ?? null : null;
  }

  function firstIssueForProject(project: V1Project | null) {
    if (!project) {
      return null;
    }
    return (
      project.issueOrder
        .flatMap((issueId) => {
          const issue = issuesById.get(issueId);
          return issue ? [issue] : [];
        })
        .sort(compareIssues)
        .at(0) ?? null
    );
  }

  return (
    <nav className="workspace-tree-nav" aria-label="工作区结构">
      <section className="workspace-tree-section">
        <div className="tree-section-header">
          <span>
            <LayoutDashboard size={17} />
            所有项目
          </span>
          <button
            className="tree-add-button sidebar-draft-button"
            aria-label="选择现有项目文件夹"
            onClick={onChooseProjectFolder}
            title="选择现有项目文件夹"
            type="button"
          >
            <FolderPlus size={17} strokeWidth={1.8} />
          </button>
        </div>

        {projectAddOpen ? (
          <form className="sidebar-project-add-panel" onSubmit={onSubmitProjectAdd}>
            <strong>添加项目</strong>
            <p>桌面客户端会打开系统文件夹选择器；浏览器预览可输入路径模拟。</p>
            <input
              aria-label="项目本地路径"
              onChange={(event) => onProjectAddPathChange(event.target.value)}
              placeholder="/Users/mac/Documents/AgentFlow"
              value={projectAddPath}
            />
            <div className="sidebar-project-add-actions">
              <button type="submit">添加</button>
              <button onClick={onCancelProjectAdd} type="button">
                取消
              </button>
            </div>
            {projectAddFeedback ? <span className="sidebar-project-add-feedback">{projectAddFeedback}</span> : null}
          </form>
        ) : projectAddFeedback ? (
          <p className="sidebar-project-add-feedback compact">{projectAddFeedback}</p>
        ) : null}

        <div className="tree-children tree-children-root tree-children-animated expanded">
          <div className="tree-children-inner">
            {projects.length === 0 ? (
              <p className="tree-empty">暂无项目</p>
            ) : (
              projects.map((project, index) => {
                const modelProject = project.modelProject;
                const firstMilestone = firstMilestoneForProject(modelProject);
                const firstIssue = firstIssueForProject(modelProject);
                const projectSelected = effectiveProjectId === project.id;
                const projectExpanded = expandedProjectIds.has(project.id);
                const milestoneActive =
                  projectSelected &&
                  activeView === "projects" &&
                  firstMilestone !== null &&
                  selectedMilestoneId === firstMilestone.id;
                const issueActive =
                  projectSelected &&
                  activeView === "issues" &&
                  selectedIssueId !== null &&
                  Boolean(modelProject?.issueOrder.includes(selectedIssueId));
                const viewActive = projectSelected && activeView === "views";
                const projectName = project.name || `项目 ${index + 1}`;
                const canRemoveProject = Boolean(project.root && project.id.startsWith("local:"));
                return (
                  <section className={projectExpanded ? "tree-nested-section expanded" : "tree-nested-section"} key={project.id}>
                    <button
                      className={projectSelected ? "tree-child tree-project-node active" : "tree-child tree-project-node"}
                      onClick={() => {
                        onSelectProject(project.id, project.root);
                        onToggleProject(project.id);
                      }}
                      aria-expanded={projectExpanded}
                      type="button"
                    >
                      <span className="tree-project-chevron" aria-hidden="true">
                        {projectExpanded ? <ChevronDown size={17} strokeWidth={2.6} /> : <ChevronRight size={17} strokeWidth={2.6} />}
                      </span>
                      <span>{projectName}</span>
                      {canRemoveProject ? (
                        <span
                          className="tree-project-remove"
                          aria-label={`从列表移除 ${projectName}`}
                          onClick={(event) => {
                            event.stopPropagation();
                            if (project.root) {
                              onRemoveProject(project.id, project.root);
                            }
                          }}
                          role="button"
                          tabIndex={0}
                          title="从列表移除，不删除源码"
                          onKeyDown={(event) => {
                            if ((event.key === "Enter" || event.key === " ") && project.root) {
                              event.preventDefault();
                              event.stopPropagation();
                              onRemoveProject(project.id, project.root);
                            }
                          }}
                        >
                          <X size={14} strokeWidth={2.2} />
                        </span>
                      ) : null}
                    </button>
                    {projectExpanded ? (
                      <div className="tree-children tree-children-animated expanded">
                        <div className="tree-children-inner">
                          <button
                            className={milestoneActive ? "tree-child tree-milestone-node active" : "tree-child tree-milestone-node"}
                            onClick={() => {
                              if (firstMilestone) {
                                onSelectMilestone(project.id, firstMilestone.id, project.root);
                              } else {
                                onSelectProject(project.id, project.root);
                              }
                            }}
                            type="button"
                          >
                            <Flag size={17} />
                            <span>里程碑</span>
                          </button>
                          <button
                            className={issueActive ? "tree-child tree-issue-node active" : "tree-child tree-issue-node"}
                            onClick={() => {
                              if (firstIssue) {
                                onSelectIssue(firstIssue.id, project.id, firstMilestone?.id ?? null, project.root);
                              } else {
                                onSelectProject(project.id, project.root);
                              }
                            }}
                            type="button"
                          >
                            <ClipboardList size={15} />
                            <span>任务</span>
                          </button>
                          <button
                            className={viewActive ? "tree-child tree-view-node active" : "tree-child tree-view-node"}
                            onClick={() => onSelectViews(project.id, project.root)}
                            type="button"
                          >
                            <LayoutDashboard size={17} />
                            <span>视图</span>
                          </button>
                        </div>
                      </div>
                    ) : null}
                  </section>
                );
              })
            )}
          </div>
        </div>
      </section>
    </nav>
  );
}

function ProjectView({
  onSelectIssue,
  onSelectMilestone,
  onSelectProject,
  onSelectProjectFile,
  projectGraphState,
  projectFilesState,
  projectViewModel,
  selectedMilestoneId,
  selectedProjectId,
  selectedProjectRoot,
}: {
  onSelectIssue: (issueId: string) => void;
  onSelectMilestone: (milestoneId: string | null) => void;
  onSelectProject: (projectId: string) => void;
  onSelectProjectFile: (relativePath: string) => void;
  projectGraphState: ProjectGraphState;
  projectFilesState: ProjectFilesState;
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null;
  selectedMilestoneId: string | null;
  selectedProjectId: string | null;
  selectedProjectRoot: string | null;
}) {
  if (!projectViewModel) {
    return <p className="empty">暂无项目 / 里程碑 / 任务 / 视图快照。</p>;
  }

  const workspace = projectViewModel.workspace;
  const sortedProjects = [...projectViewModel.projects].sort(compareProjects);
  const activeProject =
    projectViewModel.projects.find((project) => project.id === selectedProjectId) ??
    projectViewModel.projects.find((project) => project.id === workspace?.activeProjectId) ??
    sortedProjects.at(0);
  const sortedMilestones = activeProject ? [...activeProject.milestones].sort(compareMilestones) : [];
  const issuesById = new Map(projectViewModel.issues.map((issue) => [issue.id, issue]));
  const activeProjectIssues = activeProject ? issuesForProject(activeProject, issuesById) : [];
  const selectedMilestone = selectedMilestoneId
    ? sortedMilestones.find((milestone) => milestone.id === selectedMilestoneId) ?? null
    : null;
  const selectedLocalProjectRoot = selectedProjectRoot ?? projectFilesState.snapshot?.projectRoot ?? null;
  const canReadSelectedLocalProject = Boolean(selectedLocalProjectRoot);

  if (activeProject && selectedMilestone) {
    return (
      <section className="project-layout">
        <div className="runtime-hero">
          <TaskProgressCard progress={buildTaskProgress(activeProjectIssues)} />
        </div>

        <MilestoneListPanel milestones={sortedMilestones} onSelectMilestone={onSelectMilestone} selectedMilestoneId={selectedMilestone.id} />
      </section>
    );
  }

  return (
    <section className="project-layout project-local-files-layout">
      {activeProject || canReadSelectedLocalProject ? (
        <ProjectLocalFilesPage
          fileState={projectFilesState}
          graphState={projectGraphState}
          onSelectFile={onSelectProjectFile}
        />
      ) : (
        <section className="empty-project-state">
          <h2>添加项目</h2>
          <p>当前没有可展示的本地 Project。请从左侧“所有项目”添加一个现有项目文件夹。</p>
        </section>
      )}
    </section>
  );
}

function MilestoneTimeline({
  milestones,
  onSelectMilestone,
  selectedMilestoneId,
}: {
  milestones: V1Milestone[];
  onSelectMilestone?: (milestoneId: string | null) => void;
  selectedMilestoneId?: string | null;
}) {
  if (milestones.length === 0) {
    return <p className="empty milestone-timeline-empty">当前项目暂无里程碑。</p>;
  }

  return (
    <div className="milestone-timeline" aria-label="阶段时间线">
      {milestones.map((milestone, index) => {
        const previousMilestone = index > 0 ? milestones[index - 1] : null;
        const dependency = previousMilestone ? `依赖 M${index}: ${previousMilestone.name}` : "无前置阶段";
        const cardClassName = selectedMilestoneId === milestone.id ? "milestone-timeline-card active" : "milestone-timeline-card";
        const progress = milestone.progress ?? {
          doneIssueCount: 0,
          nonCanceledIssueCount: milestone.issueIds.length,
          totalIssueCount: milestone.issueIds.length,
          canceledIssueCount: 0,
          percent: 0,
        };
        const card = (
          <>
            <div className="milestone-card-main">
              <span className="milestone-step-label">M{index + 1}</span>
              <div>
                <div className="milestone-title-row">
                  <h3>{milestone.name}</h3>
                  <span>{milestone.issueIds.length} 个任务</span>
                </div>
                <p>{formatTemplateText(milestone.goal || "未记录阶段目标")}</p>
              </div>
            </div>
            <div className="milestone-card-progress" aria-label={`阶段完成度 ${formatPercent(progress.percent)}`}>
              <div>
                <StatusBadge value={displayMilestoneStatus(milestone.status)} />
                <strong>{formatMilestoneProgress(milestone)}</strong>
              </div>
              <span className="milestone-progress-track">
                <span style={{ width: `${progress.percent}%` }} />
              </span>
            </div>
            <dl className="milestone-card-facts">
              <div>
                <dt>依赖</dt>
                <dd>{dependency}</dd>
              </div>
              <div>
                <dt>证据</dt>
                <dd>{milestone.evidenceRequired.length > 0 ? "已声明" : "未记录"}</dd>
              </div>
            </dl>
          </>
        );

        return (
          <div className="milestone-timeline-item" key={milestone.id}>
            <div className="milestone-timeline-rail" aria-hidden="true">
              <span>{index + 1}</span>
            </div>
            {onSelectMilestone ? (
              <button className={cardClassName} onClick={() => onSelectMilestone(milestone.id)} type="button">
                {card}
              </button>
            ) : (
              <article className={cardClassName}>{card}</article>
            )}
          </div>
        );
      })}
    </div>
  );
}

function MilestoneTemplate({ milestone, project }: { milestone: V1Milestone; project: V1Project }) {
  const sortedMilestones = [...project.milestones].sort(compareMilestones);
  const milestoneIndex = sortedMilestones.findIndex((item) => item.id === milestone.id);
  const previousMilestone = milestoneIndex > 0 ? sortedMilestones[milestoneIndex - 1] : null;
  const dependency = previousMilestone ? `依赖 M${milestoneIndex}: ${previousMilestone.name}` : "无前置阶段";
  return (
    <article className="detail-pane task-detail-pane linear-issue-detail linear-contract-detail milestone-stage-detail">
      <header className="linear-issue-header">
        <p className="linear-issue-breadcrumb">里程碑 / {milestone.id}</p>
        <h1>{milestone.name}</h1>
        <div className="task-detail-meta-row linear-issue-properties">
          <span>项目：{project.name}</span>
          <span>阶段：{milestoneIndex >= 0 ? `M${milestoneIndex + 1}` : "未排序"}</span>
          <span>状态：{formatStatus(displayMilestoneStatus(milestone.status))}</span>
        </div>
      </header>

      <div className="linear-issue-body">
        <section className="task-detail-section linear-issue-section">
          <h3>阶段目标</h3>
          <TaskTextValue value={milestone.goal} />
        </section>

        <dl className="milestone-stage-facts">
          <div>
            <dt>依赖关系</dt>
            <dd>{dependency}</dd>
          </div>
          <div>
            <dt>包含任务</dt>
            <dd>{milestone.issueIds.length} 个</dd>
          </div>
          <div>
            <dt>完成 / 总数</dt>
            <dd>{formatMilestoneProgress(milestone)}</dd>
          </div>
          <div>
            <dt>当前状态</dt>
            <dd>{formatStatus(displayMilestoneStatus(milestone.status))}</dd>
          </div>
          <div>
            <dt>证据状态</dt>
            <dd>{milestone.evidenceRequired.length > 0 ? "已声明" : "未记录"}</dd>
          </div>
        </dl>
      </div>
    </article>
  );
}

function TemplateTextBlock({ title, value }: { title: string; value: string }) {
  return (
    <section className="task-list-group">
      <h4>{title}</h4>
      <p>{formatTemplateText(value || "未记录")}</p>
    </section>
  );
}

function TemplateInfoBlock({ title, rows }: { title: string; rows: Array<[string, string]> }) {
  return (
    <section className="list-block">
      <h3>{title}</h3>
      <ul>
        {rows.map(([label, value]) => (
          <li key={label}>
            <strong>{label}：</strong>
            {formatTemplateText(value || "未记录")}
          </li>
        ))}
      </ul>
    </section>
  );
}

function TemplateGroupedListBlock({ title, groups }: { title: string; groups: Array<[string, string[]]> }) {
  return (
    <section className="list-block">
      <h3>{title}</h3>
      <ul>
        {groups.map(([label, values]) => (
          <li key={label}>
            <strong>{label}：</strong>
            {withFallback(values).map(formatTemplateText).join("；")}
          </li>
        ))}
      </ul>
    </section>
  );
}

function MilestoneListPanel({
  milestones,
  onSelectMilestone,
  selectedMilestoneId,
}: {
  milestones: V1Milestone[];
  onSelectMilestone: (milestoneId: string | null) => void;
  selectedMilestoneId: string | null;
}) {
  return (
    <section className="project-panel">
      <div className="section-heading">
        <GitBranch size={18} />
        <h2>阶段时间线</h2>
      </div>
      <MilestoneTimeline milestones={milestones} onSelectMilestone={onSelectMilestone} selectedMilestoneId={selectedMilestoneId} />
    </section>
  );
}

function IssueListPanel({
  issues,
  onSelectIssue,
  selectedIssueId,
}: {
  issues: V1Issue[];
  onSelectIssue: (issueId: string) => void;
  selectedIssueId: string | null;
}) {
  return (
    <section className="project-panel">
      <div className="section-heading">
        <ClipboardList size={18} />
        <h2>任务列表</h2>
      </div>
      <div className="milestone-issue-list" aria-label="里程碑任务">
        {issues.length === 0 ? (
          <p className="empty compact-empty">当前范围暂无任务。</p>
        ) : (
          issues.map((issue) => (
            <button
              className={selectedIssueId === issue.id ? "milestone-issue-row active" : "milestone-issue-row"}
              key={issue.id}
              onClick={() => onSelectIssue(issue.id)}
              type="button"
            >
              <div>
                <strong>{issue.id}</strong>
                <span>{issue.title}</span>
              </div>
              <StatusBadge value={displayIssueStatus(issue)} />
            </button>
          ))
        )}
      </div>
    </section>
  );
}

function SearchView({
  query,
  searchState,
  goalLoopCommand,
  onQueryChange,
  onSearch,
}: {
  query: string;
  searchState: SearchState;
  goalLoopCommand: string;
  onQueryChange: (query: string) => void;
  onSearch: (query: string) => Promise<void>;
}) {
  const snapshot = searchState.snapshot;
  const results = snapshot?.results ?? [];
  const hasQuery = query.trim().length > 0;

  return (
    <section className="search-layout">
      <div className="search-header">
        <div>
          <p className="eyebrow">本地只读搜索</p>
          <h2>搜索</h2>
          <span>只调用 Local Search Reader，从 `.agentflow/` 事实源派生结果。</span>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      <form
        className="search-form"
        onSubmit={(event) => {
          event.preventDefault();
          void onSearch(query);
        }}
      >
        <label htmlFor="agentflow-search-query">关键词</label>
        <div className="search-input-row">
          <input
            id="agentflow-search-query"
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="输入 literal text query"
            type="search"
            value={query}
          />
          <button disabled={searchState.loading || !hasQuery} type="submit">
            {searchState.loading ? "读取中" : "搜索"}
          </button>
        </div>
        <small>不支持 regex、boolean grammar 或语义搜索；不会写 query、保存结果或执行命令。</small>
      </form>

      <section className="decision-strip compact search-command-strip">
        <div>
          <p className="eyebrow">推荐命令</p>
          <h2>只展示</h2>
          <span>桌面搜索不执行运行 / 验证 / 审查，也不创建任务。</span>
        </div>
        <div className="command-box">
          <span>目标循环推荐命令</span>
          <code>{goalLoopCommand}</code>
          <small>这里是文本展示，不绑定执行按钮。</small>
        </div>
      </section>

      {searchState.error ? (
        <div className="notice">
          <AlertTriangle size={16} />
          <span>当前无法调用 Tauri 搜索命令，未注入示例搜索结果。{searchState.error}</span>
        </div>
      ) : null}

      <div className="search-status-grid">
        <LatestMetric title="结果数量" primary={String(results.length)} detail={snapshot ? snapshot.query.query : "尚未搜索"} />
        <LatestMetric title="数据来源" primary={searchSourceLabel(searchState.source)} detail={snapshot?.projectRoot} />
        <LatestMetric title="已扫描路径" primary={String(snapshot?.searchedPaths.length ?? 0)} detail="本地授权路径" />
      </div>

      <section className="search-results">
        <div className="section-heading">
          <Search size={18} />
          <h2>搜索结果</h2>
        </div>
        {searchState.loading ? <p className="empty search-empty">正在读取本地事实源...</p> : null}
        {!searchState.loading && !snapshot ? (
          <p className="empty search-empty">输入 query 后执行只读搜索。</p>
        ) : null}
        {!searchState.loading && snapshot && results.length === 0 ? (
          <p className="empty search-empty">没有匹配结果。未写入 saved query，也未保存结果。</p>
        ) : null}
        {results.length > 0 ? (
          <div className="search-result-list">
            {results.map((result) => (
              <article className="search-result" key={`${result.path}:${result.line}:${result.snippet}`}>
                <div className="search-result-heading">
                  <div>
                    <strong>{result.path}</strong>
                    <span>
                      line {result.line} · {result.entityKind} · {result.entityId ?? "无实体 ID"}
                    </span>
                  </div>
                  <span className="score-pill">score {result.score}</span>
                </div>
                <p>{result.snippet}</p>
                <dl>
                  <div>
                    <dt>来源追踪</dt>
                    <dd>{result.sourceType}</dd>
                  </div>
                  <div>
                    <dt>字段</dt>
                    <dd>{result.field}</dd>
                  </div>
                  <div>
                    <dt>标题</dt>
                    <dd>{result.title}</dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>
        ) : null}
      </section>

      <section className="source-list">
        <h3>排除路径</h3>
        {(snapshot?.excludedPaths ?? [".agentflow/search/", ".agentflow/queries/"]).map((path) => (
          <code key={path}>{path}</code>
        ))}
      </section>
    </section>
  );
}

function MetricsView({ metrics }: { metrics: LocalMetricsSnapshot | null }) {
  if (!metrics) {
    return <p className="empty">暂无本地指标快照。</p>;
  }

  const metricGroups = [
    {
      title: "任务",
      values: [
        ["总数", metrics.issues.total],
        ["已完成", metrics.issues.completed],
        ["已计划", metrics.issues.planned],
        ["进行中", metrics.issues.active],
      ],
    },
    {
      title: "运行",
      values: [
        ["总数", metrics.runs.total],
        ["通过", metrics.runs.passed],
        ["失败", metrics.runs.failed],
        ["未验证", metrics.runs.missingValidation],
      ],
    },
    {
      title: "产物",
      values: [
        ["证据", metrics.artifacts.evidenceReports],
        ["审查", metrics.artifacts.reviews],
        ["项目更新", metrics.artifacts.projectUpdates],
        ["已保存视图", metrics.artifacts.savedViews],
      ],
    },
  ];

  return (
    <section className="metrics-layout">
      <div className="metrics-header">
        <div>
          <p className="eyebrow">本地指标快照</p>
          <h2>{metrics.projectRoot}</h2>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      <div className="metric-group-grid">
        {metricGroups.map((group) => (
          <article className="metric-group" key={group.title}>
            <h3>{group.title}</h3>
            <dl>
              {group.values.map(([label, value]) => (
                <div key={label}>
                  <dt>{label}</dt>
                  <dd>{value}</dd>
                </div>
              ))}
            </dl>
          </article>
        ))}
      </div>

      <section className="decision-strip compact">
        <div>
          <p className="eyebrow">目标循环</p>
          <h2>{formatAction(metrics.nextAction)}</h2>
          <span>
            Goal 已就绪：{formatBoolean(metrics.goalReady)} · 当前任务：{metrics.activeIssueId ?? "无"}
          </span>
        </div>
        <div className="command-box">
          <span>推荐命令</span>
          <code>{metrics.recommendedCommand}</code>
          <small>指标视图只展示命令，不会自动执行。</small>
        </div>
      </section>

      <section className="latest-grid">
        <LatestMetric title="最新运行" primary={metrics.latestRun?.id} detail={latestRunDetail(metrics)} />
        <LatestMetric
          title="最新证据"
          primary={metrics.latestEvidence?.title}
          detail={metrics.latestEvidence?.path}
        />
        <LatestMetric title="最新审查" primary={metrics.latestReview?.title} detail={metrics.latestReview?.path} />
      </section>

      <section className="source-list">
        <h3>数据来源</h3>
        {metrics.sources.map((source) => (
          <code key={source}>{source}</code>
        ))}
      </section>
    </section>
  );
}

function GoalLoopTraceView({
  snapshot,
  projectModel,
}: {
  snapshot: WorkbenchSnapshot;
  projectModel: LocalProjectModelSnapshot | null;
}) {
  const loop = snapshot.goalLoop;
  const selection = projectModel?.goalLoopSelection ?? null;
  const traceSteps = buildGoalLoopTraceSteps(loop, selection);
  const sourceEntries = Object.entries(loop?.sources ?? {});

  return (
    <section className="goal-loop-trace-layout">
      <div className="trace-header">
        <div>
          <p className="eyebrow">桌面目标循环追踪 v0</p>
          <h2>目标循环决策追踪</h2>
          <span>只读展示 `.agentflow/goal-loop.json` 和目标循环摘要；推荐命令不会在桌面端执行。</span>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      <section className="decision-strip compact trace-command-strip">
        <div>
          <p className="eyebrow">当前下一步</p>
          <h2>{formatAction(loop?.nextAction ?? "wait-human")}</h2>
          <span>{loop?.recommendedIssueIntent ?? "缺少 goal-loop.json；请在命令行运行 agentflow goal next。"}</span>
        </div>
        <div className="command-box">
          <span>推荐命令</span>
          <code>{loop?.recommendedCommand ?? "agentflow goal next"}</code>
          <small>这里没有执行按钮；用户必须回到命令行明确运行。</small>
        </div>
      </section>

      <div className="trace-status-grid">
        <LatestMetric title="目标就绪" primary={loop ? formatBoolean(loop.goalReady) : "无"} detail="未通过则等待人工确认" />
        <LatestMetric title="当前任务" primary={loop?.activeIssueId ?? "无"} detail="存在时保持单任务推进" />
        <LatestMetric
          title="未完成任务"
          primary={String(loop?.incompleteIssues.length ?? 0)}
          detail="存在时优先继续"
        />
        <LatestMetric
          title="Project candidate"
          primary={selection?.nextIssueIntent ?? "无"}
          detail={selection?.activeProjectId ?? "未读取 project model"}
        />
      </div>

      <section className="trace-priority-panel">
        <div className="section-heading">
          <ListChecks size={18} />
          <h2>决策优先级</h2>
        </div>
        <div className="trace-step-list">
          {traceSteps.map((step) => (
            <article className={`trace-step ${step.status}`} key={step.id}>
              <div>
                <strong>{step.label}</strong>
                <span>{step.detail}</span>
              </div>
              <span className="trace-step-status">{formatTraceStepStatus(step.status)}</span>
            </article>
          ))}
        </div>
      </section>

      <section className="trace-details-grid">
        <article className="trace-card">
          <div className="section-heading">
            <GitBranch size={18} />
            <h2>Project / Roadmap 来源</h2>
          </div>
          <dl className="trace-facts">
            <div>
              <dt>GoalLoopSelection source</dt>
              <dd>{selection?.source ?? "无"}</dd>
            </div>
            <div>
              <dt>Active project</dt>
              <dd>{selection?.activeProjectId ?? "无"}</dd>
            </div>
            <div>
              <dt>Project candidate</dt>
              <dd>{selection?.nextIssueIntent ?? "当前未使用 project candidate"}</dd>
            </div>
            <div>
              <dt>Roadmap fallback</dt>
              <dd>{goalLoopUsesRoadmapFallback(loop) ? "当前使用或提到 roadmap fallback" : "当前未使用 roadmap fallback"}</dd>
            </div>
          </dl>
        </article>

        <article className="trace-card">
          <div className="section-heading">
            <FileText size={18} />
            <h2>判断依据</h2>
          </div>
          {loop?.rationale.length ? (
            <ul className="trace-rationale-list">
              {loop.rationale.map((item) => (
                <li key={item}>{localizeGoalLoopRationale(item)}</li>
              ))}
            </ul>
          ) : (
            <p className="empty trace-card-empty">暂无 rationale。运行 `agentflow goal next` 后生成。</p>
          )}
        </article>
      </section>

      <section className="split-grid">
        <MarkdownPanel
          fallback="缺少 .agentflow/updates/GOAL-LOOP-SUMMARY.md"
          icon={GitBranch}
          title="目标循环摘要"
          value={snapshot.goalLoopSummaryMarkdown}
        />
        <article className="trace-card trace-source-card">
          <div className="section-heading">
            <FileSearch size={18} />
            <h2>数据来源</h2>
          </div>
          <div className="trace-source-list">
            {sourceEntries.length === 0 ? (
              <p className="empty">暂无 sources。</p>
            ) : (
              sourceEntries.map(([key, value]) => (
                <div key={key}>
                  <span>{formatGoalLoopSourceKey(key)}</span>
                  <code>{value}</code>
                </div>
              ))
            )}
          </div>
        </article>
      </section>
    </section>
  );
}

function Overview({ projectModel }: { projectModel: LocalProjectModelSnapshot | null }) {
  return (
    <div className="view-stack">
      <WorkspaceOverview projectModel={projectModel} />
    </div>
  );
}

function WorkspaceOverview({ projectModel }: { projectModel: LocalProjectModelSnapshot | null }) {
  const workspace = projectModel?.workspace;

  if (!projectModel || !workspace) {
    return (
      <section className="workspace-overview">
        <div className="workspace-overview-heading">
          <div>
            <p className="eyebrow">项目</p>
            <h2>尚未读取本地项目</h2>
          </div>
          <span className="readonly-tag">只读</span>
        </div>
        <p className="empty">运行本地项目读取器后，这里会显示本地项目列表。</p>
      </section>
    );
  }

  const projects = workspace.projectIds
    .map((projectId) => projectModel.projects.find((project) => project.id === projectId))
    .filter((project): project is NonNullable<typeof project> => Boolean(project));
  const sortedProjects = [...projects].sort(compareProjects);

  return (
    <section className="workspace-overview">
      <div className="workspace-overview-heading">
        <div>
          <p className="eyebrow">项目入口</p>
          <h2>本地项目</h2>
          <span>系统默认上下文已隐藏</span>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      <div className="workspace-entry-grid">
        <section className="workspace-entry-column">
          <div className="section-heading compact-heading">
            <FolderKanban size={18} />
            <h2>项目列表</h2>
          </div>
          <div className="workspace-project-list">
            {sortedProjects.length === 0 ? (
              <p className="empty">暂无项目。</p>
            ) : (
              sortedProjects.map((project) => (
                <article className="workspace-project-row" key={project.id}>
                  <div>
                    <strong>{project.name}</strong>
                    <span>{project.id}</span>
                  </div>
                  <StatusBadge value={displayProjectStatus(project)} />
                  <small>
                    {project.milestones.length} 个里程碑 · {project.completedIssueCount}/{project.issueCount} 任务完成
                  </small>
                </article>
              ))
            )}
          </div>
        </section>
      </div>
    </section>
  );
}

function TeamView({
  createOpen,
  onDismissCreate,
  onSelectTeam,
  projectModel,
  selectedTeamId,
}: {
  createOpen: boolean;
  onDismissCreate: () => void;
  onSelectTeam: (teamId: string) => void;
  projectModel: LocalProjectModelSnapshot | null;
  selectedTeamId: string | null;
}) {
  const workspace = projectModel?.workspace;

  if (!projectModel || !workspace) {
    return <p className="empty">暂无本地团队模型快照。</p>;
  }

  const projects = [...projectModel.projects].sort(compareProjects);
  const teams = [...projectModel.teams].sort(compareTeams);
  const issueRefsById = new Map(projectModel.issueRefs.map((issue) => [issue.id, issue]));
  const selectedTeam =
    teams.find((team) => team.id === selectedTeamId) ?? teams.at(0) ?? null;
  const selectedTeamProjects = selectedTeam
    ? projects.filter((project) => project.teamIds.includes(selectedTeam.id)).sort(compareProjects)
    : [];
  const selectedTeamIssues = selectedTeam
    ? selectedTeam.issueIds.flatMap((issueId) => {
        const issue = issueRefsById.get(issueId);
        return issue ? [issue] : [];
      }).sort(compareIssues)
    : [];

  return (
    <section className="team-layout">
      <div className="team-header">
        <div>
          <p className="eyebrow">团队</p>
          <h2>{workspace.name}</h2>
          <span>{teams.length} 个团队 · {projects.length} 个项目</span>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      {createOpen ? (
        <section className="team-create-panel" aria-label="新增团队">
          <div>
            <p className="eyebrow">新增团队</p>
            <h3>初始化创建入口</h3>
            <span>这里是团队创建入口占位。当前不会写 `.agentflow/teams/`，后续需要 Team Writer 合同后再保存。</span>
          </div>
          <button className="team-create-dismiss" onClick={onDismissCreate} type="button">
            关闭
          </button>
        </section>
      ) : null}

      <div className="team-relation-grid" aria-label="团队父子栏目">
        <section className="team-relation-column team-parent-column">
          <div className="team-column-heading">
            <span>父级栏目</span>
            <h3>团队</h3>
            <small>{teams.length} 个团队</small>
          </div>

          <div className="team-column-list">
            {teams.length === 0 ? (
              <p className="empty">工作区下暂无团队。</p>
            ) : (
              teams.map((team) => {
                const teamProjects = projects.filter((project) => project.teamIds.includes(team.id));
                const active = selectedTeam?.id === team.id;
                return (
                  <button
                    className={`team-parent-card ${active ? "active" : ""}`}
                    key={team.id}
                    type="button"
                    onClick={() => onSelectTeam(team.id)}
                  >
                    <UsersRound size={17} />
                    <span>
                      <strong>{team.name}</strong>
                      <small>{team.id}</small>
                    </span>
                    <em>
                      {teamProjects.length} 项目 / {team.issueIds.length} 任务
                    </em>
                  </button>
                );
              })
            )}
          </div>
        </section>

        <section className="team-relation-column">
          <div className="team-column-heading">
            <span>子级栏目</span>
            <h3>项目</h3>
            <small>{selectedTeam ? `隶属于 ${selectedTeam.name}` : "先选择团队"}</small>
          </div>
          <div className="team-column-list">
            {selectedTeamProjects.length === 0 ? (
              <p className="empty">当前团队暂无关联项目。</p>
            ) : (
              selectedTeamProjects.map((project) => (
                <div className="team-child-card" key={project.id}>
                  <FolderKanban size={16} />
                  <span>
                    <strong>{project.name}</strong>
                    <small>{project.id}</small>
                  </span>
                  <StatusBadge value={displayProjectStatus(project)} />
                </div>
              ))
            )}
          </div>
        </section>

        <section className="team-relation-column">
          <div className="team-column-heading">
            <span>子级栏目</span>
            <h3>任务</h3>
            <small>{selectedTeam ? `隶属于 ${selectedTeam.name}` : "先选择团队"}</small>
          </div>
          <div className="team-column-list">
            {selectedTeamIssues.length === 0 ? (
              <p className="empty">当前团队暂无关联任务。</p>
            ) : (
              selectedTeamIssues.map((issue) => (
                <div className="team-child-card" key={issue.id}>
                  <ClipboardList size={16} />
                  <span>
                    <strong>{issue.id}</strong>
                    <small>{issue.title}</small>
                  </span>
                  <StatusBadge value={displayIssueStatus(issue)} />
                </div>
              ))
            )}
          </div>
        </section>
      </div>
    </section>
  );
}

function LatestMetric({ title, primary, detail }: { title: string; primary?: string; detail?: string }) {
  return (
    <article className="latest-metric">
      <p className="eyebrow">{title}</p>
      <strong>{primary ?? "无"}</strong>
      <span>{detail ?? "未记录"}</span>
    </article>
  );
}

function IssueLifecycleTraceView({
  issues,
  issueRuns,
  selectedIssue,
  onSelectIssue,
  evidence,
  reviews,
  projectUpdates,
}: {
  issues: IssueContract[];
  issueRuns: Map<string, AgentRun[]>;
  selectedIssue: IssueContract | null;
  onSelectIssue: (issueId: string) => void;
  evidence: WorkbenchTextArtifact[];
  reviews: WorkbenchTextArtifact[];
  projectUpdates: WorkbenchTextArtifact[];
}) {
  const runs = selectedIssue ? issueRuns.get(selectedIssue.id) ?? [] : [];
  const trace = selectedIssue ? buildIssueLifecycleTrace(selectedIssue, runs, evidence, reviews, projectUpdates) : null;

  return (
    <section className="lifecycle-layout">
      <div className="lifecycle-header">
        <div>
          <p className="eyebrow">Desktop Issue Lifecycle Trace v0</p>
          <h2>Issue 生命周期追踪</h2>
          <span>只读展示 contract、run、validation、evidence、review、project update 和 completed 状态。</span>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      <section className="lifecycle-content">
        <div className="lifecycle-issue-list">
          <div className="section-heading">
            <ClipboardList size={18} />
            <h2>Issue Contracts</h2>
          </div>
          {issues.length === 0 ? (
            <p className="empty lifecycle-empty">暂无 issue contract。</p>
          ) : (
            issues.map((issue) => {
              const issueTrace = buildIssueLifecycleTrace(
                issue,
                issueRuns.get(issue.id) ?? [],
                evidence,
                reviews,
                projectUpdates,
              );
              return (
                <button
                  className={selectedIssue?.id === issue.id ? "lifecycle-issue-row active" : "lifecycle-issue-row"}
                  key={issue.id}
                  onClick={() => onSelectIssue(issue.id)}
                  type="button"
                >
                  <span>
                    <strong>{issue.id}</strong>
                    <small>{issue.title}</small>
                  </span>
                  <StatusBadge value={displayIssueStatus(issue)} />
                  <small>{issueTrace.currentStep}</small>
                </button>
              );
            })
          )}
        </div>

        <article className="lifecycle-detail">
          {selectedIssue && trace ? (
            <>
              <div className="detail-heading">
                <div>
                  <p className="eyebrow">{selectedIssue.id}</p>
                  <h2>{selectedIssue.title}</h2>
                </div>
                <StatusBadge value={displayIssueStatus(selectedIssue)} />
              </div>

              <section className="decision-strip compact lifecycle-command-strip">
                <div>
                  <p className="eyebrow">当前生命周期步骤</p>
                  <h2>{trace.currentStep}</h2>
                  <span>{trace.currentDetail}</span>
                </div>
                <div className="command-box">
                  <span>桌面边界</span>
                  <code>read-only lifecycle trace</code>
                  <small>没有执行按钮；不会触发 run / verify / review。</small>
                </div>
              </section>

              <div className="lifecycle-step-list">
                {trace.steps.map((step) => (
                  <article className={`lifecycle-step ${step.status}`} key={step.id}>
                    <div>
                      <strong>{step.label}</strong>
                      <span>{step.detail}</span>
                    </div>
                    <span className="lifecycle-step-status">{formatLifecycleStatus(step.status)}</span>
                  </article>
                ))}
              </div>

              <section className="lifecycle-panels">
                <div className="lifecycle-panel">
                  <div className="section-heading compact-heading">
                    <FileText size={18} />
                    <h2>Contract</h2>
                  </div>
                  <div className="lifecycle-panel-body">
                    <ListBlock title="范围" values={selectedIssue.scope} />
                    <ListBlock title="非目标" values={selectedIssue.nonGoals} />
                  </div>
                </div>

                <div className="lifecycle-panel">
                  <div className="section-heading compact-heading">
                    <ListChecks size={18} />
                    <h2>Validation</h2>
                  </div>
                  <div className="lifecycle-panel-body">
                    {trace.latestRun ? (
                      <>
                        <div className="run-card">
                          <span>{trace.latestRun.id}</span>
                          <StatusBadge value={trace.latestRun.status} />
                          <StatusBadge value={validationStatus(trace.latestRun)} />
                          <small>{trace.latestRun.mode}</small>
                        </div>
                        <div className="validation-command-list">
                          {trace.latestRun.validationCommands.length === 0 ? (
                            <p className="empty">暂无 validation command。</p>
                          ) : (
                            trace.latestRun.validationCommands.map((command) => (
                              <div key={command.command}>
                                <code>{command.command}</code>
                                <StatusBadge value={command.exitCode === 0 ? "passed" : "failed"} />
                              </div>
                            ))
                          )}
                        </div>
                      </>
                    ) : (
                      <p className="empty">暂无 run，生命周期停在 contract 后。</p>
                    )}
                  </div>
                </div>
              </section>

              <section className="lifecycle-artifact-grid">
                <LifecycleArtifactList title="Evidence" artifacts={trace.evidenceArtifacts} paths={trace.evidencePaths} />
                <LifecycleArtifactList title="Review" artifacts={trace.reviewArtifacts} paths={trace.reviewPaths} />
                <LifecycleArtifactList title="Project Update" artifacts={trace.updateArtifacts} paths={trace.updatePaths} />
              </section>
            </>
          ) : (
            <p className="empty">选择一个 issue 查看生命周期 trace。</p>
          )}
        </article>
      </section>
    </section>
  );
}

function IssuesView({
  projectViewModel,
  selectedIssueId,
  selectedMilestoneId,
  selectedProjectId,
  onSelectIssue,
}: {
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null;
  selectedIssueId: string | null;
  selectedMilestoneId: string | null;
  selectedProjectId: string | null;
  onSelectIssue: (issueId: string) => void;
}) {
  const activeProject = selectedProjectFromV1(projectViewModel, selectedProjectId);
  const scopedIssues = scopeV1IssuesForSelection(projectViewModel?.issues ?? [], activeProject, selectedMilestoneId);
  const sortedIssues = [...scopedIssues].sort(compareIssues);
  const issuesById = new Map((projectViewModel?.issues ?? []).map((issue) => [issue.id, issue]));
  const projectIssues = activeProject ? issuesForProject(activeProject, issuesById) : sortedIssues;
  const taskProgress = buildTaskProgress(projectIssues);
  const activeIssue =
    selectedIssueId && sortedIssues.some((issue) => issue.id === selectedIssueId)
      ? sortedIssues.find((issue) => issue.id === selectedIssueId) ?? null
      : sortedIssues.at(0) ?? null;

	  return (
	    <section className="issue-page">
	      <div className="runtime-hero">
	        <TaskProgressCard progress={taskProgress} />
	      </div>

      <section className="issue-layout template-detail-layout">
        <IssueListPanel issues={sortedIssues} onSelectIssue={onSelectIssue} selectedIssueId={activeIssue?.id ?? null} />
        <IssueTemplate issue={activeIssue} />
      </section>
    </section>
  );
}

function IssueTemplate({ issue }: { issue: V1Issue | null }) {
  return <IssueDetail issue={issue} />;
}

function TaskProgressCard({ progress }: { progress: TaskProgressSnapshot }) {
  return (
    <div className="task-progress-card">
      <div className="task-progress-heading">
        <span>完成进度</span>
        <strong>{formatPercent(progress.donePercent)}</strong>
      </div>
      <div className="task-progress-track" aria-label={`任务完成度 ${formatPercent(progress.donePercent)}`}>
        <TaskProgressSegment percent={progress.donePercent} variant="done" />
        <TaskProgressSegment percent={progress.activePercent} variant="active" />
        <TaskProgressSegment percent={progress.pendingPercent} variant="pending" />
        <TaskProgressSegment percent={progress.canceledPercent} variant="canceled" />
      </div>
      <div className="task-progress-stats">
        <TaskProgressStat label="已完成" value={progress.done} variant="done" />
        <TaskProgressStat label="进行中" value={progress.active} variant="active" />
        <TaskProgressStat label="待处理" value={progress.pending} variant="pending" />
        <TaskProgressStat label="已取消" value={progress.canceled} variant="canceled" />
        <TaskProgressStat label="总计" value={progress.total} variant="total" />
      </div>
    </div>
  );
}

function TaskProgressSegment({ percent, variant }: { percent: number; variant: string }) {
  if (percent <= 0) {
    return null;
  }
  return <span className={`task-progress-segment ${variant}`} style={{ width: `${percent}%` }} />;
}

function TaskProgressStat({
  label,
  value,
  variant,
}: {
  label: string;
  value: number;
  variant: string;
}) {
  return (
    <div className={`task-progress-stat ${variant}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function IssueDetail({ issue }: { issue: V1Issue | null }) {
  if (!issue) {
    return (
      <article className="detail-pane">
        <p className="empty">选择一个任务，查看它的执行卡片。</p>
      </article>
    );
  }

  const validationCommands = mergeUnique(["git diff --check"], issue.validationCommands);
  const evidenceRequirements = withFallback(issue.evidenceRequired, [
    "摘要",
    "变更文件",
    "运行命令",
    "命令结果",
    "新增 / 更新测试",
    "行为影响",
    "回滚计划",
  ]);
  const boundaryRules = uniqueByComparableText(mergeUnique(issue.boundary, [
    "不扩大范围",
    "不修改禁止文件",
    "不自动推进下一个任务",
  ]));

  return (
    <article className="detail-pane issue-contract-page">
      <header className="issue-contract-header">
        <div>
          <p>任务 / {issue.id}</p>
          <h1>{issue.title}</h1>
        </div>
        <div className="issue-contract-properties">
          <span>状态：{formatStatus(displayIssueStatus(issue))}</span>
          <span>风险：{formatRiskLevel(issue.riskLevel)}</span>
          <span>里程碑：{issue.milestoneId ?? "未分配 / 项目级任务"}</span>
        </div>
      </header>

      <div className="issue-contract-body">
        <section className="issue-contract-hero">
          <span>目标</span>
          <p>{formatTemplateText(issue.goal || issue.title)}</p>
        </section>

        <div className="issue-contract-section-stack">
          <IssueContractPanel title="范围" values={withFallback(issue.scope)} />
          <IssueContractPanel title="可能文件" values={withFallback(issue.allowedFiles)} code />
          <IssueContractPanel title="非目标" values={withFallback(issue.nonGoals)} />
          <IssueContractPanel title="依赖" values={withFallback(issue.dependencies, ["无前置任务"])} />
          <IssueContractPanel title="验收标准" values={withFallback(issue.acceptanceCriteria, ["<可验证结果>"])} checklist />
          <IssueContractPanel title="验证命令" values={validationCommands} code />
          <IssueContractPanel title="证据要求" values={evidenceRequirements} />
          <IssueContractPanel title="边界" values={boundaryRules} emphasis />
        </div>
      </div>
    </article>
  );
}

function IssueContractPanel({
  checklist = false,
  code = false,
  emphasis = false,
  title,
  values,
}: {
  checklist?: boolean;
  code?: boolean;
  emphasis?: boolean;
  title: string;
  values: string[];
}) {
  return (
    <section className={emphasis ? "issue-contract-panel emphasis" : "issue-contract-panel"}>
      <h3>{title}</h3>
      <ul className={checklist ? "issue-contract-checklist" : undefined}>
        {values.map((value) => (
          <li key={value}>
            {checklist ? <span aria-hidden="true" /> : null}
            {code ? <code>{formatTemplateText(value)}</code> : formatTemplateText(value)}
          </li>
        ))}
      </ul>
    </section>
  );
}

function uniqueByComparableText(values: string[]): string[] {
  const seen = new Set<string>();
  const uniqueValues: string[] = [];

  for (const value of values) {
    const key = value.trim().replace(/[。.\s]/g, "").toLowerCase();
    if (!key || seen.has(key)) {
      continue;
    }
    seen.add(key);
    uniqueValues.push(value);
  }

  return uniqueValues;
}

function TaskTextValue({ value }: { value: string }) {
  return <p>{formatTemplateText(value || "未记录")}</p>;
}

function TaskValueList({ values, code = false }: { values: string[]; code?: boolean }) {
  if (values.length === 0) {
    return <p className="task-muted">无</p>;
  }

  return (
    <ul className="task-value-list">
      {values.map((value) => (
        <li key={value}>{code ? <code>{formatTemplateText(value)}</code> : formatTemplateText(value)}</li>
      ))}
    </ul>
  );
}

function TaskTextBlock({ title, value }: { title: string; value: string }) {
  return (
    <section className="task-list-group">
      <h4>{title}</h4>
      <p>{formatTemplateText(value || "未记录")}</p>
    </section>
  );
}

function TaskListGroup({ title, values, code = false }: { title: string; values: string[]; code?: boolean }) {
  return (
    <section className="task-list-group">
      <h4>{title}</h4>
      {values.length === 0 ? (
        <p className="task-muted">无</p>
      ) : (
        <ul>
          {values.map((value) => (
            <li key={value}>{code ? <code>{formatTemplateText(value)}</code> : formatTemplateText(value)}</li>
          ))}
        </ul>
      )}
    </section>
  );
}

function TaskOrderedBlock({ title, values }: { title: string; values: string[] }) {
  return (
    <section className="task-list-group">
      <h4>{title}</h4>
      <TaskOrderedList values={values} />
    </section>
  );
}

function TaskOrderedList({ values }: { values: string[] }) {
  if (values.length === 0) {
    return <p className="task-muted">暂无实现指南。</p>;
  }

  return (
    <ol className="task-guide-list">
      {values.map((value) => (
        <li key={value}>{formatTemplateText(value)}</li>
      ))}
    </ol>
  );
}

function ProjectUpdateTimelineView({ snapshot }: { snapshot: WorkbenchSnapshot }) {
  const timeline = useMemo(() => buildProjectUpdateTimeline(snapshot), [snapshot]);

  return (
    <section className="project-update-timeline-layout">
      <div className="timeline-header">
        <div>
          <p className="eyebrow">Desktop Project Update Timeline v0</p>
          <h2>项目更新时间线</h2>
          <span>只读展示 PROJECT-UPDATE、issue、run、evidence 和 review 的本地推进链路。</span>
        </div>
        <span className="readonly-tag">只读</span>
      </div>

      <section className="decision-strip compact timeline-command-strip">
        <div>
          <p className="eyebrow">项目推进链路</p>
          <h2>Issue -&gt; Run -&gt; Validation -&gt; Evidence -&gt; Review -&gt; Project Update</h2>
          <span>每条 update 都从本地 `.agentflow/` 事实源派生关联关系。</span>
        </div>
        <div className="command-box">
          <span>桌面边界</span>
          <code>read-only project update timeline</code>
          <small>没有执行按钮；不会创建 issue、执行 run / verify / review 或保存筛选条件。</small>
        </div>
      </section>

      <section className="timeline-summary-grid" aria-label="项目更新时间线计数">
        <LatestMetric title="Project Updates" primary={String(timeline.length)} detail="最新优先" />
        <LatestMetric title="Issues" primary={String(snapshot.issues.length)} detail={`${snapshot.counts.completedIssues} 已完成`} />
        <LatestMetric title="Runs" primary={String(snapshot.runs.length)} detail={`${snapshot.counts.passedRuns} 已通过`} />
        <LatestMetric title="Evidence / Review" primary={`${snapshot.evidence.length} / ${snapshot.reviews.length}`} detail="只读链接" />
      </section>

      <section className="timeline-board">
        <div className="section-heading">
          <History size={18} />
          <h2>Project Update Timeline</h2>
        </div>
        {timeline.length === 0 ? (
          <p className="empty timeline-empty">暂无 PROJECT-UPDATE 记录。</p>
        ) : (
          <div className="timeline-list">
            {timeline.map((item) => (
              <article className="timeline-item" key={item.update.path}>
                <div className="timeline-marker">
                  <strong>{item.order}</strong>
                  <span>{item.updateId}</span>
                </div>
                <div className="timeline-card">
                  <div className="timeline-card-heading">
                    <div>
                      <p className="eyebrow">{item.update.path}</p>
                      <h3>{item.update.title}</h3>
                    </div>
                    <StatusBadge value={item.status} />
                  </div>

                  <p>{item.snippet}</p>

                  <div className="timeline-link-grid">
                    <TimelineRefList title="Issue Contract" refs={item.issueRefs} />
                    <TimelineRefList title="Run" refs={item.runRefs} />
                    <TimelineRefList title="Validation" refs={item.validationRefs} />
                    <TimelineRefList title="Evidence" refs={item.evidenceRefs} />
                    <TimelineRefList title="Review" refs={item.reviewRefs} />
                    <TimelineRefList title="Project Update" refs={[{ id: item.updateId, detail: item.update.path }]} />
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}

function ArtifactView({
  artifacts,
  selectedArtifact,
  onSelectArtifact,
  title,
  emptyLabel,
}: {
  artifacts: WorkbenchTextArtifact[];
  selectedArtifact: WorkbenchTextArtifact | null;
  onSelectArtifact: (path: string) => void;
  title: string;
  emptyLabel: string;
}) {
  const activeArtifact = selectedArtifact && artifacts.some((artifact) => artifact.path === selectedArtifact.path)
    ? selectedArtifact
    : artifacts.at(-1) ?? null;

  return (
    <section className="artifact-layout">
      <div className="artifact-list">
        <div className="section-heading">
          <FileSearch size={18} />
          <h2>{title}</h2>
        </div>
        {artifacts.length === 0 ? (
          <p className="empty">{emptyLabel}</p>
        ) : (
          artifacts.map((artifact) => (
            <button
              className={activeArtifact?.path === artifact.path ? "artifact-row active" : "artifact-row"}
              key={artifact.path}
              onClick={() => onSelectArtifact(artifact.path)}
              type="button"
            >
              <strong>{artifact.title}</strong>
              <span>{artifact.path}</span>
            </button>
          ))
        )}
      </div>
      <article className="artifact-reader">
        {activeArtifact ? (
          <>
            <div className="detail-heading">
              <div>
                <p className="eyebrow">{activeArtifact.path}</p>
                <h2>{activeArtifact.title}</h2>
              </div>
              <span className="readonly-tag">只读</span>
            </div>
            <pre>{activeArtifact.content}</pre>
          </>
        ) : (
          <p className="empty">{emptyLabel}</p>
        )}
      </article>
    </section>
  );
}

function SavedViewsView({
  onSelectView,
  selectedViewId,
  views,
}: {
  onSelectView: (viewId: string) => void;
  selectedViewId: string | null;
  views: V1View[];
}) {
  const sortedViews = [...views].sort(compareViews);
  const activeView = sortedViews.find((view) => view.id === selectedViewId) ?? sortedViews.at(0) ?? null;

  return (
    <section className="saved-view-layout">
      <div className="saved-view-hero">
        <div>
          <p>项目和任务</p>
          <h2>保存视图</h2>
          <span>View 只是 saved filter，不写业务状态，也不执行命令。</span>
        </div>
        <strong>{views.length} 个视图</strong>
      </div>

      <section className="saved-view-workbench">
        <aside className="saved-view-list" aria-label="保存视图列表">
          {sortedViews.length === 0 ? (
            <p className="empty">暂无保存视图。</p>
          ) : (
            sortedViews.map((view) => (
              <button
                className={activeView?.id === view.id ? "saved-view active" : "saved-view"}
                key={view.id}
                onClick={() => onSelectView(view.id)}
                type="button"
              >
                <span>{view.id}</span>
                <strong>{view.name}</strong>
                <small>{formatViewLayout(view.layout)}</small>
              </button>
            ))
          )}
        </aside>

        <article className="saved-view-detail" aria-label="保存视图详情">
          {activeView ? (
            <>
              <header>
                <p>{activeView.id}</p>
                <h2>{activeView.name}</h2>
              </header>
              <dl className="saved-view-rule-grid">
                {Object.entries(activeView.filter).map(([key, value]) => (
                  <div key={key}>
                    <dt>{formatFilterKey(key)}</dt>
                    <dd>{value ? formatStatus(value) : "任意"}</dd>
                  </div>
                ))}
                <div>
                  <dt>排序</dt>
                  <dd>{activeView.sort.map(formatViewSort).join(" / ") || "默认"}</dd>
                </div>
                <div>
                  <dt>布局</dt>
                  <dd>{formatViewLayout(activeView.layout)}</dd>
                </div>
              </dl>
              <p className="saved-view-note">只读保存筛选器，不写业务状态，也不执行命令。</p>
            </>
          ) : (
            <p className="empty">选择一个保存视图。</p>
          )}
        </article>
      </section>
    </section>
  );
}

function MarkdownPanel({
  title,
  value,
  fallback,
  icon: Icon,
}: {
  title: string;
  value?: string | null;
  fallback: string;
  icon: LucideIcon;
}) {
  return (
    <article className="markdown-panel">
      <div className="section-heading">
        <Icon size={18} />
        <h2>{title}</h2>
      </div>
      <pre>{localizeGeneratedMarkdown(value?.trim() || fallback)}</pre>
    </article>
  );
}

function BoundaryPanel({ snapshot }: { snapshot: WorkbenchSnapshot }) {
  return (
    <section className="boundary-panel">
      <div className="boundary-title">
        {snapshot.boundary.readOnly ? <CheckCircle2 size={16} /> : <AlertTriangle size={16} />}
        <strong>{snapshot.boundary.readOnly ? "浏览本地事实源" : "本地写入模式"}</strong>
      </div>
      <dl className="boundary-summary">
        <div>
          <dt>可执行动作</dt>
          <dd>查看本地事实源、复制 CLI 命令、查看证据。</dd>
        </div>
        <div>
          <dt>禁止动作</dt>
          <dd>直接执行、直接写入、远程创建。</dd>
        </div>
      </dl>
    </section>
  );
}

function ListBlock({ title, values, code = false }: { title: string; values: string[]; code?: boolean }) {
  return (
    <section className="list-block">
      <h3>{title}</h3>
      {values.length === 0 ? (
        <p className="empty">无</p>
      ) : (
        <ul>
          {values.map((value) => (
            <li key={value}>{code ? <code>{formatTemplateText(value)}</code> : formatTemplateText(value)}</li>
          ))}
        </ul>
      )}
    </section>
  );
}

function SidebarFooterLinks() {
  return (
    <nav className="sidebar-footer-links" aria-label="辅助入口">
      <button type="button">
        <Settings size={15} />
        <span>Settings</span>
      </button>
      <button type="button">
        <FileText size={15} />
        <span>Docs</span>
      </button>
    </nav>
  );
}

function localProjectIdFromRoot(projectRoot: string) {
  return `local:${normalizeProjectRootKey(projectRoot)}`;
}

function localProjectFolderFromRoot(projectRoot: string, metadata?: unknown): LocalProjectFolder {
  const name = projectNameFromPath(projectRoot) || projectRoot;
  const metadataObject = metadata && typeof metadata === "object" ? metadata : null;
  const agentflowPath =
    metadataObject && "agentflowPath" in metadataObject && typeof metadataObject.agentflowPath === "string"
      ? metadataObject.agentflowPath
      : null;
  const gitProtected =
    metadataObject && "protectedGitExclude" in metadataObject && typeof metadataObject.protectedGitExclude === "boolean"
      ? metadataObject.protectedGitExclude
      : metadataObject && "gitProtected" in metadataObject && typeof metadataObject.gitProtected === "boolean"
        ? metadataObject.gitProtected
        : false;
  const preparedAt =
    metadataObject && "preparedAt" in metadataObject && typeof metadataObject.preparedAt === "string"
      ? metadataObject.preparedAt
      : null;

  return {
    id: localProjectIdFromRoot(projectRoot),
    name,
    root: projectRoot,
    agentflowPath,
    gitProtected,
    preparedAt,
  };
}

function upsertLocalProjectFolder(projects: LocalProjectFolder[], nextProject: LocalProjectFolder) {
  const existingIndex = projects.findIndex((project) => projectRootsEqual(project.root, nextProject.root));
  if (existingIndex < 0) {
    return [...projects, nextProject];
  }
  return projects.map((project, index) => (index === existingIndex ? nextProject : project));
}

function removeLocalProjectFolderByRoot(projects: LocalProjectFolder[], projectRoot: string) {
  return projects.filter((project) => !projectRootsEqual(project.root, projectRoot));
}

function projectWorkspaceFeedback(summary: ProjectWorkspaceSummary, prefix: string) {
  const agentflowState = summary.createdAgentflow ? "已创建 .agentflow/" : "已复用 .agentflow/";
  const gitState = summary.gitExcludePath
    ? summary.protectedGitExclude
      ? "已保护 Git exclude"
      : "Git exclude 未更新"
    : "未检测到 Git 仓库";
  return `${prefix}（${agentflowState}，${gitState}）`;
}

function findModelProjectForRoot(modelProjects: V1Project[], projectRoot: string, workspaceProjectRoot?: string | null) {
  if (!projectRootsEqual(projectRoot, workspaceProjectRoot)) {
    return null;
  }
  const projectName = projectNameFromPath(projectRoot);
  return modelProjects.find((project) => project.name === projectName) ?? null;
}

function buildSidebarProjectItems(
  modelProjects: V1Project[],
  localProjectFolders: LocalProjectFolder[],
  workspaceProjectRoot?: string | null,
): SidebarProjectItem[] {
  const workspaceProject = workspaceProjectRoot
    ? modelProjects.find((project) => project.name === projectNameFromPath(workspaceProjectRoot))
    : null;
  const workspaceRootWasAssigned = Boolean(workspaceProject && workspaceProjectRoot);
  const filteredLocalProjectFolders =
    workspaceRootWasAssigned && workspaceProjectRoot
      ? removeLocalProjectFolderByRoot(localProjectFolders, workspaceProjectRoot)
      : localProjectFolders;

  return [
    ...modelProjects.map((project) => ({
      id: project.id,
      name: project.name,
      root: workspaceProject?.id === project.id ? workspaceProjectRoot ?? null : null,
      modelProject: project,
    })),
    ...filteredLocalProjectFolders.map((project) => ({
      id: project.id,
      name: project.name,
      root: project.root,
      modelProject: null,
    })),
  ];
}


function formatTemplateText(value: string) {
  const normalized = value
    .replaceAll("Workspace / Team / Project /", "Project /")
    .replaceAll("Workspace / Team /", "")
    .replaceAll("Workspace / Team", "系统默认上下文")
    .replaceAll("Default Workspace", "系统默认上下文")
    .replace(/\bWorkspace\b/g, "系统默认上下文")
    .replace(/\bTeam\b/g, "系统默认上下文")
    .trim();
  return normalized || "未记录";
}

function withFallback(values: string[], fallback: string[] = ["未记录"]) {
  const normalized = values.map((value) => value.trim()).filter(Boolean);
  return normalized.length > 0 ? normalized : fallback;
}

function mergeUnique(primary: string[], secondary: string[]) {
  const values = [...primary, ...secondary].map((value) => value.trim()).filter(Boolean);
  return Array.from(new Set(values));
}

function projectHasTasks(project: V1Project) {
  return project.issueOrder.length > 0 || project.milestones.some((milestone) => milestone.issueIds.length > 0);
}

function issuesForProject(project: V1Project, issuesById: ReadonlyMap<string, V1Issue>) {
  const issueIds = uniqueStrings([...project.issueOrder, ...project.milestones.flatMap((milestone) => milestone.issueIds)]);
  return issueIds.flatMap((issueId) => {
    const issue = issuesById.get(issueId);
    return issue ? [issue] : [];
  });
}

function issuesForMilestone(milestone: V1Milestone, issuesById: ReadonlyMap<string, V1Issue>) {
  return milestone.issueIds.flatMap((issueId) => {
    const issue = issuesById.get(issueId);
    return issue ? [issue] : [];
  });
}

function LifecycleArtifactList({
  title,
  paths,
  artifacts,
}: {
  title: string;
  paths: string[];
  artifacts: WorkbenchTextArtifact[];
}) {
  return (
    <section className="lifecycle-artifact-list">
      <h3>{title}</h3>
      {paths.length === 0 && artifacts.length === 0 ? (
        <p className="empty">暂无链接。</p>
      ) : (
        <>
          {uniqueStrings([...paths, ...artifacts.map((artifact) => artifact.path)]).map((path) => (
            <code key={path}>{path}</code>
          ))}
          {artifacts.slice(0, 1).map((artifact) => (
            <pre key={artifact.path}>{artifact.content.trim().slice(0, 520)}</pre>
          ))}
        </>
      )}
    </section>
  );
}

function TimelineRefList({ title, refs }: { title: string; refs: TimelineRef[] }) {
  return (
    <section className="timeline-ref-list">
      <h4>{title}</h4>
      {refs.length === 0 ? (
        <p className="empty">无</p>
      ) : (
        refs.map((ref) => (
          <div key={`${title}:${ref.id}:${ref.detail}`}>
            <strong>{ref.id}</strong>
            <span>{ref.detail}</span>
            {ref.status ? <StatusBadge value={ref.status} /> : null}
          </div>
        ))
      )}
    </section>
  );
}

function StatusBadge({ value }: { value: string }) {
  return <span className={`status-badge ${statusClassName(value)}`}>{formatStatus(value)}</span>;
}

function statusClassName(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-");
}

const PROJECT_STATUS_ORDER = ["active", "draft", "completed", "canceled", "paused"];
const ISSUE_STATUS_ORDER = ["in_progress", "in_review", "todo", "backlog", "done", "canceled"];
const MILESTONE_FLOW_ORDER = ["project-charter", "milestone-plan", "issue-contracts", "validation-evidence"];

function compareProjects<T extends { id: string; name: string; status: string; canonicalStatus?: string | null }>(
  left: T,
  right: T,
) {
  return (
    statusRank(displayProjectStatus(left), PROJECT_STATUS_ORDER) - statusRank(displayProjectStatus(right), PROJECT_STATUS_ORDER) ||
    left.name.localeCompare(right.name, "zh-Hans") ||
    left.id.localeCompare(right.id)
  );
}

function compareIssues<T extends { id: string; title: string; status: string; canonicalStatus?: string | null }>(
  left: T,
  right: T,
) {
  return (
    statusRank(displayIssueStatus(left), ISSUE_STATUS_ORDER) - statusRank(displayIssueStatus(right), ISSUE_STATUS_ORDER) ||
    issueNumber(left.id) - issueNumber(right.id) ||
    left.title.localeCompare(right.title, "zh-Hans") ||
    left.id.localeCompare(right.id)
  );
}

function compareTeams<T extends { id: string; name: string }>(left: T, right: T) {
  return left.name.localeCompare(right.name, "zh-Hans") || left.id.localeCompare(right.id);
}

function compareMilestones<T extends { id: string; name: string; sortOrder?: number | null }>(left: T, right: T) {
  return (
    milestoneFlowRank(left) - milestoneFlowRank(right) ||
    left.name.localeCompare(right.name, "zh-Hans") ||
    left.id.localeCompare(right.id)
  );
}

function milestoneFlowRank(milestone: { id: string; name: string; sortOrder?: number | null }) {
  if (typeof milestone.sortOrder === "number") {
    return milestone.sortOrder;
  }

  const normalizedId = milestone.id.toLowerCase();
  const normalizedName = milestone.name.toLowerCase();
  const directIndex = MILESTONE_FLOW_ORDER.indexOf(normalizedId);
  if (directIndex >= 0) {
    return directIndex;
  }
  if (normalizedId.includes("project-charter") || normalizedName.includes("project charter") || normalizedName.includes("项目目标")) {
    return 0;
  }
  if (normalizedId.includes("milestone-plan") || normalizedName.includes("milestone plan") || normalizedName.includes("里程碑")) {
    return 1;
  }
  if (normalizedId.includes("issue-contract") || normalizedName.includes("issue contract") || normalizedName.includes("任务合同")) {
    return 2;
  }
  if (
    normalizedId.includes("validation") ||
    normalizedId.includes("evidence") ||
    normalizedName.includes("validation") ||
    normalizedName.includes("evidence") ||
    normalizedName.includes("验证") ||
    normalizedName.includes("证据")
  ) {
    return 3;
  }
  return Number.MAX_SAFE_INTEGER;
}

function compareViews(left: V1View, right: V1View) {
  return left.name.localeCompare(right.name, "zh-Hans") || left.id.localeCompare(right.id);
}

function sortedV1Issues(issues: V1Issue[]) {
  return [...issues].sort(compareIssues);
}

function selectedProjectFromV1(projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null, selectedProjectId: string | null) {
  if (!projectViewModel) {
    return null;
  }
  const sortedProjects = [...projectViewModel.projects].sort(compareProjects);
  return (
    projectViewModel.projects.find((project) => project.id === selectedProjectId) ??
    projectViewModel.projects.find((project) => project.id === projectViewModel.workspace?.activeProjectId) ??
    sortedProjects.at(0) ??
    null
  );
}

function scopeV1IssuesForSelection(issues: V1Issue[], activeProject: V1Project | null, selectedMilestoneId: string | null) {
  if (!activeProject) {
    return sortedV1Issues(issues);
  }
  const milestone = activeProject.milestones.find((item) => item.id === selectedMilestoneId);
  const issueIds = new Set(milestone?.issueIds.length ? milestone.issueIds : activeProject.issueOrder);
  if (issueIds.size === 0) {
    return [];
  }
  return sortedV1Issues(issues.filter((issue) => issueIds.has(issue.id)));
}

function issueBelongsToTeam(
  issue: V1Issue,
  teamId: string,
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null,
) {
  if (!issue.projectId || !projectViewModel) {
    return false;
  }
  const team = projectViewModel.teams.find((item) => item.id === teamId);
  if (team?.issueIds.includes(issue.id)) {
    return true;
  }
  const project = projectViewModel.projects.find((item) => item.id === issue.projectId);
  return Boolean(team?.projectIds.includes(project?.id ?? ""));
}

function filterAndSortIssues<T extends { id: string; title: string; status: string; goal?: string; intent?: string }>(
  issues: T[],
  query: string,
  statusFilter: string,
  sortMode: string,
) {
  const normalizedQuery = query.trim().toLowerCase();
  return issues
    .filter((issue) => {
      const status = displayIssueStatus(issue);
      const matchesStatus = statusFilter === "all" || status === statusFilter;
      const matchesQuery =
        normalizedQuery.length === 0 ||
        `${issue.id} ${issue.title} ${issue.goal ?? ""} ${issue.intent ?? ""}`.toLowerCase().includes(normalizedQuery);
      return matchesStatus && matchesQuery;
    })
    .sort((left, right) => compareIssuesByMode(left, right, sortMode));
}

function compareIssuesByMode<T extends { id: string; title: string; status: string; canonicalStatus?: string | null }>(
  left: T,
  right: T,
  sortMode: string,
) {
  if (sortMode === "id-asc") {
    return issueNumber(left.id) - issueNumber(right.id) || left.id.localeCompare(right.id);
  }
  if (sortMode === "id-desc") {
    return issueNumber(right.id) - issueNumber(left.id) || right.id.localeCompare(left.id);
  }
  if (sortMode === "title-asc") {
    return left.title.localeCompare(right.title, "zh-Hans") || left.id.localeCompare(right.id);
  }
  if (sortMode === "title-desc") {
    return right.title.localeCompare(left.title, "zh-Hans") || left.id.localeCompare(right.id);
  }
  return compareIssues(left, right);
}

function buildIssueStatusCounts<T extends { status: string; canonicalStatus?: string | null }>(issues: T[]) {
  return issues.reduce<Record<string, number>>((counts, issue) => {
    const status = displayIssueStatus(issue);
    counts[status] = (counts[status] ?? 0) + 1;
    return counts;
  }, {});
}

function buildTaskProgress<T extends { status: string; canonicalStatus?: string | null }>(issues: T[]): TaskProgressSnapshot {
  const counts = buildIssueStatusCounts(issues);
  const total = issues.length;
  const done = counts.done ?? 0;
  const active = (counts.in_progress ?? 0) + (counts.in_review ?? 0);
  const pending = (counts.backlog ?? 0) + (counts.todo ?? 0);
  const canceled = counts.canceled ?? 0;
  return {
    total,
    done,
    active,
    pending,
    canceled,
    donePercent: percentOf(done, total),
    activePercent: percentOf(active, total),
    pendingPercent: percentOf(pending, total),
    canceledPercent: percentOf(canceled, total),
  };
}

function percentOf(value: number, total: number) {
  if (total === 0) {
    return 0;
  }
  return Math.round((value / total) * 1000) / 10;
}

function statusRank(value: string, order: string[]) {
  const index = order.indexOf(value);
  return index === -1 ? order.length : index;
}

function issueNumber(issueId: string) {
  const match = issueId.match(/\d+/);
  return match ? Number(match[0]) : Number.MAX_SAFE_INTEGER;
}

function displayProjectStatus(project: { status: string; canonicalStatus?: string | null }) {
  return project.canonicalStatus ?? canonicalProjectStatus(project.status);
}

function displayIssueStatus(issue: { status: string; canonicalStatus?: string | null }) {
  return issue.canonicalStatus ?? canonicalIssueStatus(issue.status);
}

function canonicalProjectStatus(value: string) {
  const normalized = normalizeStatus(value);
  if (["active", "audit", "docs_refresh", "final_review", "closing"].includes(normalized)) return "active";
  if (["planned", "ready", "confirmed"].includes(normalized)) return "draft";
  if (["paused", "blocked", "failed"].includes(normalized)) return "paused";
  if (["completed", "done"].includes(normalized)) return "completed";
  if (["canceled", "cancelled"].includes(normalized)) return "canceled";
  return "draft";
}

function canonicalIssueStatus(value: string) {
  const normalized = normalizeStatus(value);
  if (["todo", "planned", "ready"].includes(normalized)) return "todo";
  if (["in_progress", "active", "eligible", "leased"].includes(normalized)) return "in_progress";
  if (["in_review", "review", "pr", "checks_passing", "merged", "evidence_captured", "needs_human_review"].includes(normalized)) {
    return "in_review";
  }
  if (["done", "completed"].includes(normalized)) return "done";
  if (["canceled", "cancelled"].includes(normalized)) return "canceled";
  return "backlog";
}

function normalizeStatus(value: string) {
  return value.trim().toLowerCase().replaceAll("-", "_").replace(/\s+/g, "_");
}

function displayMilestoneStatus(value: string) {
  const normalized = normalizeStatus(value);
  if (["active", "in_progress", "eligible", "leased"].includes(normalized)) return "in_progress";
  if (normalized === "review") return "in_review";
  if (["ready", "todo", "planned"].includes(normalized)) return "todo";
  if (normalized === "done" || normalized === "completed") return "done";
  if (normalized === "canceled" || normalized === "cancelled") return "canceled";
  if (["blocked", "failed", "repair"].includes(normalized)) return "backlog";
  return "backlog";
}

function formatMilestoneProgress(milestone: {
  issueIds: string[];
  completedIssueIds?: string[];
  progress?: MilestoneDerivedProgress;
}) {
  const progress = milestone.progress ?? {
    doneIssueCount: milestone.completedIssueIds?.length ?? 0,
    nonCanceledIssueCount: milestone.issueIds.length,
    totalIssueCount: milestone.issueIds.length,
    canceledIssueCount: 0,
    percent: milestone.issueIds.length === 0 ? 0 : Math.floor(((milestone.completedIssueIds?.length ?? 0) * 100) / milestone.issueIds.length),
  };
  return `${progress.doneIssueCount}/${progress.nonCanceledIssueCount} 任务完成`;
}

function mapRunsByIssue(runs: AgentRun[]) {
  return runs.reduce((map, run) => {
    const group = map.get(run.issueId) ?? [];
    group.push(run);
    group.sort((left, right) => left.id.localeCompare(right.id));
    map.set(run.issueId, group);
    return map;
  }, new Map<string, AgentRun[]>());
}

function validationStatus(run: AgentRun) {
  if (run.validationCommands.length === 0) {
    return "not-recorded";
  }
  return run.validationCommands.every((command) => command.exitCode === 0) ? "passed" : "failed";
}

function latestRunDetail(metrics: LocalMetricsSnapshot) {
  if (!metrics.latestRun) {
    return undefined;
  }
  return `${metrics.latestRun.issueId} · ${formatStatus(metrics.latestRun.status)} / ${formatStatus(metrics.latestRun.validationStatus)}`;
}

function localizeGeneratedMarkdown(value: string) {
  const replacements: Array<[RegExp, string]> = [
    [/^# Project Summary$/gm, "# 项目摘要"],
    [/^# Goal Loop Summary$/gm, "# 目标循环摘要"],
    [/^## Counts$/gm, "## 计数"],
    [/^## Next Issue$/gm, "## 下一个任务"],
    [/^## Issues$/gm, "## 任务"],
    [/^## Runs$/gm, "## 运行"],
    [/^## Saved Views$/gm, "## 保存视图"],
    [/^## Incomplete Issues$/gm, "## 未完成任务"],
    [/^## Rationale$/gm, "## 判断依据"],
    [/^## Boundary$/gm, "## 边界"],
    [/\bGenerated:/g, "生成时间："],
    [/\bExecutor:/g, "执行者："],
    [/\bSQLite index:/g, "SQLite 索引："],
    [/\bGoal ready:/g, "Goal 已就绪："],
    [/\bActive issue:/g, "当前任务："],
    [/\bNext action:/g, "下一步："],
    [/\bRecommended intent:/g, "推荐意图："],
    [/\bRecommended command:/g, "推荐命令："],
    [/\bGoal Loop is local-decision-only\./g, "目标循环只做本地决策。"],
    [
      /\bIt does not execute code, create remote issues, call models, or bypass IssueContract\./g,
      "它不会执行代码、创建远程 issue、调用模型或绕过 IssueContract。",
    ],
    [/\| Item \| Count \|/g, "| 项目 | 数量 |"],
    [/\| Issue \| Status \| Title \|/g, "| 任务 | 状态 | 标题 |"],
    [/\| Run \| Issue \| Status \| Validation \|/g, "| 运行 | 任务 | 状态 | 验证 |"],
    [/\| View \| Issue status \| Run status \| Validation \|/g, "| 视图 | 任务状态 | 运行状态 | 验证 |"],
    [/\| Issue \| Status \| Next action \| Title \|/g, "| 任务 | 状态 | 下一步 | 标题 |"],
    [/\bCompleted issues\b/g, "已完成任务"],
    [/\bPlanned issues\b/g, "已计划任务"],
    [/\bEvidence reports\b/g, "证据报告"],
    [/\bProject updates\b/g, "项目更新"],
    [/\bSaved views\b/g, "保存视图"],
    [/\bPassed runs\b/g, "已通过运行"],
    [/\bIssues\b/g, "任务"],
    [/\bRuns\b/g, "运行"],
    [/\bReviews\b/g, "审查"],
    [/\bnone\b/g, "无"],
    [/\bcompleted\b/g, "已完成"],
    [/\bplanned\b/g, "已计划"],
    [/\bpassed\b/g, "已通过"],
    [/\bfailed\b/g, "失败"],
  ];
  return replacements.reduce((content, [pattern, replacement]) => content.replace(pattern, replacement), value);
}

function formatBoolean(value: boolean) {
  return value ? "是" : "否";
}

function formatPercent(value: number) {
  return `${Number.isInteger(value) ? value.toFixed(0) : value.toFixed(1)}%`;
}

function formatAction(value: string) {
  const actions: Record<string, string> = {
    plan: "规划",
    run: "运行",
    verify: "验证",
    review: "审查",
    update: "更新摘要",
    "wait-human": "等待人工确认",
  };
  return actions[value] ?? value;
}

function formatStatus(value: string) {
  const statuses: Record<string, string> = {
    active: "进行中",
    any: "任意",
    backlog: "待整理",
    canceled: "已取消",
    completed: "已完成",
    done: "已完成",
    failed: "失败",
    draft: "草稿",
    in_progress: "进行中",
    in_review: "审查中",
    "not-recorded": "未记录",
    passed: "已通过",
    planned: "已计划",
    paused: "暂停",
    review: "审查",
    run: "运行",
    todo: "待办",
    "wait-human": "等待人工",
    verify: "验证",
  };
  return statuses[value] ?? value;
}

function formatRiskLevel(value: string) {
  const risks: Record<string, string> = {
    low: "低",
    medium: "中",
    high: "高",
  };
  const normalized = normalizeStatus(value || "");
  return risks[normalized] ?? "未记录";
}

function formatFilterKey(value: string) {
  const keys: Record<string, string> = {
    issueId: "任务 ID",
    issueStatus: "任务状态",
    runStatus: "运行状态",
    validationStatus: "验证状态",
  };
  return keys[value] ?? value;
}

function formatViewSort(sort: V1ViewSort) {
  const fields: Record<string, string> = {
    id: "ID",
    issueId: "任务 ID",
    issueStatus: "任务状态",
    status: "状态",
    title: "标题",
    updatedAt: "更新时间",
    createdAt: "创建时间",
  };
  const directions: Record<string, string> = {
    asc: "升序",
    desc: "降序",
  };
  return `${fields[sort.field] ?? sort.field} ${directions[sort.direction] ?? sort.direction}`;
}

function formatViewLayout(value: string) {
  const layouts: Record<string, string> = {
    list: "列表",
    board: "看板",
    timeline: "时间线",
    table: "表格",
  };
  return layouts[value] ?? value;
}

function formatBoundaryAction(value: string) {
  const actions: Record<string, string> = {
    "create-issue": "创建任务",
    run: "执行运行",
    verify: "执行验证",
    review: "执行审查",
    "model-call": "调用模型",
    "write-agentflow-facts": "写入 .agentflow/ 事实源",
    "remote-pr-or-issue": "创建远程 PR / issue",
  };
  return actions[value] ?? value;
}

type LifecycleStepStatus = "done" | "current" | "missing" | "failed";

type TimelineRef = {
  id: string;
  detail: string;
  status?: string;
};

type ProjectUpdateTimelineItem = {
  order: string;
  updateId: string;
  status: string;
  snippet: string;
  update: WorkbenchTextArtifact;
  issueRefs: TimelineRef[];
  runRefs: TimelineRef[];
  validationRefs: TimelineRef[];
  evidenceRefs: TimelineRef[];
  reviewRefs: TimelineRef[];
};

function buildProjectUpdateTimeline(snapshot: WorkbenchSnapshot): ProjectUpdateTimelineItem[] {
  const issuesById = new Map(snapshot.issues.map((issue) => [issue.id, issue]));
  const runsById = new Map(snapshot.runs.map((run) => [run.id, run]));

  return [...snapshot.projectUpdates]
    .sort((left, right) => projectUpdateSortKey(right).localeCompare(projectUpdateSortKey(left)))
    .map((update, index) => {
      const issueIds = uniqueStrings(extractIds(update.content, /ISSUE-\d{4}/g));
      const runIds = uniqueStrings(extractIds(update.content, /RUN-\d{4}/g));
      const linkedRuns = uniqueById([
        ...runIds.map((runId) => runsById.get(runId)).filter(Boolean),
        ...snapshot.runs.filter(
          (run) => run.outputs.update === update.path || issueIds.includes(run.issueId) || update.content.includes(run.issueId),
        ),
      ] as AgentRun[]);
      const linkedIssueIds = uniqueStrings([...issueIds, ...linkedRuns.map((run) => run.issueId)]);
      const issueRefs = linkedIssueIds.map((issueId) => {
        const issue = issuesById.get(issueId);
        return {
          id: issueId,
          detail: issue?.title ?? "未找到本地 issue contract",
          status: issue?.status,
        };
      });
      const runRefs = linkedRuns.map((run) => ({
        id: run.id,
        detail: `${run.issueId} · ${run.mode}`,
        status: run.status,
      }));
      const validationRefs = linkedRuns.map((run) => ({
        id: run.id,
        detail: `${run.validationCommands.length} 条命令`,
        status: validationStatus(run),
      }));
      const evidenceRefs = artifactRefs(
        snapshot.evidence,
        uniqueStrings([
          ...extractArtifactPaths(update.content, ".agentflow/evidence/"),
          ...(linkedRuns.map((run) => run.outputs.evidence).filter(Boolean) as string[]),
        ]),
        linkedIssueIds,
      );
      const reviewRefs = artifactRefs(
        snapshot.reviews,
        uniqueStrings([
          ...extractArtifactPaths(update.content, ".agentflow/reviews/"),
          ...(linkedRuns.map((run) => run.outputs.review).filter(Boolean) as string[]),
        ]),
        linkedIssueIds,
      );

      return {
        order: `#${String(index + 1).padStart(2, "0")}`,
        updateId: projectUpdateId(update),
        status: projectUpdateStatus(update),
        snippet: projectUpdateSnippet(update.content),
        update,
        issueRefs,
        runRefs,
        validationRefs,
        evidenceRefs,
        reviewRefs,
      };
    });
}

function projectUpdateSortKey(update: WorkbenchTextArtifact) {
  return projectUpdateId(update).padStart(24, "0");
}

function projectUpdateId(update: WorkbenchTextArtifact) {
  return extractIds(`${update.path}\n${update.title}`, /PROJECT-UPDATE-\d{4}/g).at(0) ?? update.title;
}

function projectUpdateStatus(update: WorkbenchTextArtifact) {
  const match = update.content.match(/Status:\s*`?([A-Za-z0-9_-]+)`?/);
  return match?.[1] ?? "not-recorded";
}

function projectUpdateSnippet(content: string) {
  const summary = content.match(/## Summary\s+([\s\S]*?)(\n## |\n# |$)/)?.[1]?.trim();
  return (summary || content.replace(/^# .+$/m, "").trim()).slice(0, 260) || "无摘要。";
}

function artifactRefs(artifacts: WorkbenchTextArtifact[], linkedPaths: string[], issueIds: string[]) {
  const linked = new Set(linkedPaths);
  return artifacts
    .filter(
      (artifact) =>
        linked.has(artifact.path) ||
        issueIds.some((issueId) => artifact.path.includes(issueId) || artifact.content.includes(issueId)),
    )
    .map((artifact) => ({
      id: artifact.title,
      detail: artifact.path,
    }));
}

function extractIds(content: string, pattern: RegExp) {
  return Array.from(content.matchAll(pattern), (match) => match[0]);
}

function extractArtifactPaths(content: string, prefix: string) {
  return Array.from(content.matchAll(new RegExp(`${escapeRegExp(prefix)}[^\\x60\\s)]+`, "g")), (match) => match[0]);
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function uniqueById(runs: AgentRun[]) {
  return Array.from(new Map(runs.map((run) => [run.id, run])).values()).sort((left, right) => left.id.localeCompare(right.id));
}

type IssueLifecycleStep = {
  id: string;
  label: string;
  detail: string;
  status: LifecycleStepStatus;
};

type IssueLifecycleTrace = {
  steps: IssueLifecycleStep[];
  currentStep: string;
  currentDetail: string;
  latestRun: AgentRun | null;
  evidenceArtifacts: WorkbenchTextArtifact[];
  reviewArtifacts: WorkbenchTextArtifact[];
  updateArtifacts: WorkbenchTextArtifact[];
  evidencePaths: string[];
  reviewPaths: string[];
  updatePaths: string[];
};

function buildIssueLifecycleTrace(
  issue: IssueContract,
  runs: AgentRun[],
  evidence: WorkbenchTextArtifact[],
  reviews: WorkbenchTextArtifact[],
  projectUpdates: WorkbenchTextArtifact[],
): IssueLifecycleTrace {
  const sortedRuns = [...runs].sort((left, right) => left.id.localeCompare(right.id));
  const latestRun = sortedRuns.at(-1) ?? null;
  const validation = latestRun ? validationStatus(latestRun) : "not-recorded";
  const evidencePaths = uniqueStrings(sortedRuns.map((run) => run.outputs.evidence).filter(Boolean) as string[]);
  const reviewPaths = uniqueStrings(sortedRuns.map((run) => run.outputs.review).filter(Boolean) as string[]);
  const updatePaths = uniqueStrings(sortedRuns.map((run) => run.outputs.update).filter(Boolean) as string[]);
  const evidenceArtifacts = matchingIssueArtifacts(issue.id, evidence, evidencePaths);
  const reviewArtifacts = matchingIssueArtifacts(issue.id, reviews, reviewPaths);
  const updateArtifacts = matchingIssueArtifacts(issue.id, projectUpdates, updatePaths);
  const hasEvidence = evidencePaths.length > 0 || evidenceArtifacts.length > 0;
  const hasReview = reviewPaths.length > 0 || reviewArtifacts.length > 0;
  const hasUpdate = updatePaths.length > 0 || updateArtifacts.length > 0;
  const steps: IssueLifecycleStep[] = [
    {
      id: "contract",
      label: "Contract",
      detail: `${issue.id} 已存在本地 IssueContract。`,
      status: "done",
    },
    {
      id: "run",
      label: "Run",
      detail: latestRun ? `最新 run 为 ${latestRun.id} / ${formatStatus(latestRun.status)}。` : "尚未生成 run。",
      status: latestRun ? "done" : "current",
    },
    {
      id: "validation",
      label: "Validation",
      detail: latestRun
        ? `${latestRun.validationCommands.length} 条 validation command，当前 ${formatStatus(validation)}。`
        : "等待 run 之后记录 validation。",
      status: !latestRun ? "missing" : validation === "passed" ? "done" : validation === "failed" ? "failed" : "current",
    },
    {
      id: "evidence",
      label: "Evidence",
      detail: hasEvidence ? "已关联 evidence report。" : "尚未关联 evidence report。",
      status: hasEvidence ? "done" : latestRun ? "current" : "missing",
    },
    {
      id: "review",
      label: "Review",
      detail: hasReview ? "已关联 review / assistant 文档。" : "尚未关联 review 文档。",
      status: hasReview ? "done" : hasEvidence ? "current" : "missing",
    },
    {
      id: "project-update",
      label: "Project Update",
      detail: hasUpdate ? "已关联 project update。" : "尚未关联 project update。",
      status: hasUpdate ? "done" : hasReview ? "current" : "missing",
    },
    {
      id: "completed",
      label: "Completed",
      detail: canonicalIssueStatus(issue.status) === "done" ? "Issue 已完成。" : `Issue 当前状态为 ${formatStatus(displayIssueStatus(issue))}。`,
      status: canonicalIssueStatus(issue.status) === "done" ? "done" : hasUpdate ? "current" : "missing",
    },
  ];
  const current = steps.find((step) => step.status === "failed" || step.status === "current") ?? steps.at(-1);

  return {
    steps,
    currentStep: current?.label ?? "Unknown",
    currentDetail: current?.detail ?? "没有生命周期状态。",
    latestRun,
    evidenceArtifacts,
    reviewArtifacts,
    updateArtifacts,
    evidencePaths,
    reviewPaths,
    updatePaths,
  };
}

function matchingIssueArtifacts(issueId: string, artifacts: WorkbenchTextArtifact[], linkedPaths: string[]) {
  const linked = new Set(linkedPaths);
  return artifacts.filter(
    (artifact) => linked.has(artifact.path) || artifact.path.includes(issueId) || artifact.content.includes(issueId),
  );
}

function uniqueStrings(values: string[]) {
  return Array.from(new Set(values.filter((value) => value.trim().length > 0)));
}

function formatLifecycleStatus(value: LifecycleStepStatus) {
  const statuses: Record<LifecycleStepStatus, string> = {
    current: "当前",
    done: "完成",
    failed: "失败",
    missing: "等待",
  };
  return statuses[value];
}

type TraceStepStatus = "passed" | "blocked" | "current" | "skipped" | "fallback";

type TraceStep = {
  id: string;
  label: string;
  detail: string;
  status: TraceStepStatus;
};

function buildGoalLoopTraceSteps(loop: GoalLoopState | null | undefined, selection: GoalLoopSelection | null): TraceStep[] {
  const incompleteCount = loop?.incompleteIssues.length ?? 0;
  const hasActiveIssue = Boolean(loop?.activeIssueId);
  const usesProjectCandidate = goalLoopUsesProjectCandidate(loop);
  const usesRoadmapFallback = goalLoopUsesRoadmapFallback(loop);
  const projectCandidate = selection?.nextIssueIntent ?? null;

  return [
    {
      id: "readiness",
      label: "1. 目标就绪",
      detail: loop ? `当前为 ${formatBoolean(loop.goalReady)}；未通过时返回 wait-human。` : "缺少 goal-loop.json。",
      status: loop?.goalReady ? "passed" : "blocked",
    },
    {
      id: "active-issue",
      label: "2. 当前任务 / 单任务推进",
      detail: hasActiveIssue ? `当前任务为 ${loop?.activeIssueId}，必须先完成。` : "没有当前任务。",
      status: hasActiveIssue ? "current" : "passed",
    },
    {
      id: "incomplete-issue",
      label: "3. Incomplete issue",
      detail: incompleteCount > 0 ? `${incompleteCount} 个未完成 issue 优先于新规划。` : "没有未完成 issue。",
      status: incompleteCount > 0 ? "current" : "passed",
    },
    {
      id: "project-candidate",
      label: "4. Active project / milestone candidate",
      detail: projectCandidate
        ? `${selection?.activeProjectId ?? "active project"} 推荐：${projectCandidate}`
        : "当前没有可用 project candidate，或被 active / incomplete issue 阻塞。",
      status: usesProjectCandidate ? "current" : projectCandidate ? "skipped" : "skipped",
    },
    {
      id: "roadmap-fallback",
      label: "5. Roadmap candidate fallback",
      detail: usesRoadmapFallback ? "当前回退到 roadmap / construction plan candidate。" : "当前未使用 roadmap fallback。",
      status: usesRoadmapFallback ? "fallback" : "skipped",
    },
    {
      id: "wait-human",
      label: "6. Wait human",
      detail: loop?.nextAction === "wait-human" ? "当前需要人工补齐事实源或处理异常。" : "当前不需要人工等待。",
      status: loop?.nextAction === "wait-human" ? "blocked" : "skipped",
    },
  ];
}

function goalLoopUsesProjectCandidate(loop: GoalLoopState | null | undefined) {
  return (loop?.rationale ?? []).some((item) => item.includes("active project candidate") || item.includes("Active project"));
}

function goalLoopUsesRoadmapFallback(loop: GoalLoopState | null | undefined) {
  return (loop?.rationale ?? []).some((item) => item.includes("roadmap") || item.includes("Roadmap"));
}

function formatTraceStepStatus(value: TraceStepStatus) {
  const statuses: Record<TraceStepStatus, string> = {
    blocked: "阻塞",
    current: "当前命中",
    fallback: "回退命中",
    passed: "通过",
    skipped: "未命中",
  };
  return statuses[value];
}

function formatGoalLoopSourceKey(value: string) {
  const keys: Record<string, string> = {
    goal: "Goal",
    index: "Index",
    projectDefinition: "Project Definition",
    projectSummary: "Project Summary",
    roadmap: "Roadmap",
    scopeState: "Scope State",
  };
  return keys[value] ?? value;
}

function localizeGoalLoopRationale(value: string) {
  const replacements: Array<[RegExp, string]> = [
    [/Goal readiness is ready\./g, "目标就绪检查已通过。"],
    [/Goal readiness is not ready\./g, "目标就绪检查未通过。"],
    [/No active issue is set\./g, "当前没有 active issue。"],
    [/All known issues are completed; Goal Loop can use the active project candidate\./g, "所有已知任务已完成，目标循环可使用当前项目候选项。"],
    [/All known issues are completed; Goal Loop can recommend the next roadmap intent\./g, "所有已知任务已完成，目标循环回退推荐下一条路线图意图。"],
    [/Goal Loop must finish the current issue before recommending new planning\./g, "目标循环必须先完成当前任务，不能开始新规划。"],
    [/Roadmap does not authorize a new issue while an existing issue remains open\./g, "仍有未完成 issue 时，roadmap 不授权创建新 issue。"],
  ];
  return replacements.reduce((content, [pattern, replacement]) => content.replace(pattern, replacement), value);
}

function searchSourceLabel(value: SearchState["source"]) {
  const sources: Record<SearchState["source"], string> = {
    idle: "未搜索",
    preview: "浏览器 Mock 数据",
    unavailable: "本地搜索不可用",
    tauri: "桌面真实数据",
  };
  return sources[value];
}

export default App;
