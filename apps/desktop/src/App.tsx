import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  CheckCircle2,
  ClipboardCheck,
  ClipboardList,
  FileSearch,
  FolderOpen,
  GitBranch,
  LayoutDashboard,
  ListChecks,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  ChevronDown,
  ChevronRight,
  Plus,
  X,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState, type MouseEvent, type ReactNode } from "react";
import { detectAppLocale } from "./appLocale";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewAuditIndex,
  createBrowserPreviewHumanAuditReport,
  createBrowserPreviewOutputIndex,
  createBrowserPreviewProjectViewModelSnapshot,
  createBrowserPreviewWorkbenchSnapshot,
} from "./browserPreviewData";
import {
  ActionButton,
  AppFrame,
  CopyableCodeBlock,
  MetricCard,
  PageHeader,
  Panel,
  RiskBadge,
  Section,
  Sidebar,
  StatusBadge,
  StatusBar as FoundationStatusBar,
  TopBar,
  WindowChrome,
  type StatusChipStatus,
} from "./components";
import { useAgentManual } from "./features/agent-manual";
import { useExecuteStatus } from "./features/execute";
import { useInputSnapshot, useInputStatus } from "./features/input";
import { useOutputStatus, type OutputStatusState } from "./features/output";
import {
  ProjectLocalFilesPage,
  isBrowserPreviewRuntime,
  normalizeProjectRootKey,
  projectNameFromPath,
  useProjectFiles,
  useProjectPanel,
  type ProjectPanelState,
  type ProjectFilesState,
} from "./features/project-files";
import { useIssueStatusIndex, useStateStatus, type StateStatusState } from "./features/state";
import {
  createBrowserPreviewProjectRegistry,
  createProjectRef,
  isAgentFlowProjectPage,
  persistProjectRegistry,
  projectRegistryStorageKeys,
  readProjectRegistry,
  removeProject,
  selectProject,
  setProjectPage,
  toggleProjectExpanded,
  upsertProject,
  type AgentFlowProjectPage,
  type AgentFlowProjectRef,
  type AgentFlowProjectStatus,
} from "./projectRegistry";
import {
  buildAppInteractionState,
  buildAuditInteractionState,
  buildDeliveryInteractionState,
  buildTaskInteractionState,
  buildTaskProjectTreeViewModel,
  displayStatusLabelZh,
  pickTaskId,
  taskActionLabel,
  type AppInteractionState,
  type ButtonInteractionState,
  type TaskInteractionAction,
  type TaskIssueNode,
  type TaskProjectGroup,
  type TaskProjectTreeViewModel,
} from "./interaction/viewModels";
import type {
  AuditIndex,
  AuditIndexEntry,
  ExecutionPipeline,
  HumanAuditReport,
  AgentRole,
  InputIssue,
  IssueDisplayStatus,
  IssueStatusIndex,
  IssueContract,
  OutputIndex,
  OutputIndexEntry,
  ProjectMilestoneIssueViewModelSnapshot,
  V1Issue,
  WorkbenchSnapshot,
  ExpectedOutputs,
} from "./types";
import "./AppShell.css";

type Provider = "ChatGPT" | "Claude" | "DeepSeek";
type AppPage = AgentFlowProjectPage;
type DataSource = "idle" | "loading" | "tauri" | "preview" | "unavailable";

type WorkspaceDataState = {
  error: string | null;
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null;
  source: DataSource;
  workbench: WorkbenchSnapshot | null;
};

type OutputBundleState = {
  auditIndex: AuditIndex | null;
  auditReport: HumanAuditReport | null;
  error: string | null;
  outputIndex: OutputIndex | null;
  source: DataSource;
};

type ProjectInitializationContext = {
  author?: string | null;
  changedFiles: string[];
  committedAt?: string | null;
  id: string;
  sourceUrl?: string | null;
  summary: string;
  title: string;
};

type ProjectInitializationStatus = {
  demoAuditCount: number;
  demoDataCreated: boolean;
  demoDeliveryCount: number;
  demoIssueCount: number;
  gitContextLoaded: boolean;
  initialized: boolean;
  message: string;
  paths: string[];
  projectKind: "new" | "existing" | string;
  recentContext: ProjectInitializationContext[];
  recentContextCount: number;
  version: string;
  warnings: string[];
};

type ProjectInitializationState = {
  error: string | null;
  source: DataSource;
  status: ProjectInitializationStatus | null;
};

type ProjectWorkspaceSummary = {
  initializationStatus?: ProjectInitializationStatus | null;
};

type AgentflowWorkspaceChangedEvent = {
  agentflowPath: string;
  changedAreas: string[];
  eventKind: string;
  paths: string[];
  projectRoot: string;
  updatedAt: number;
  version: string;
  watcherBackend: string;
  watcherStatus: string;
};

type WorkflowEventDispatchSummary = {
  contextPackFailed: number;
  contextPackReady: number;
  contextPackRequests: number;
  emittedIssueReadyEvents: number;
  errors: string[];
  pendingPanelEvents: number;
  version: string;
};

const AGENTFLOW_WORKSPACE_CHANGED_EVENT = "agentflow-workspace-changed";
const AGENTFLOW_WATCHER_REFRESH_DELAY_MS = 500;
const AGENTFLOW_WATCHER_REFRESH_COOLDOWN_MS = 1200;
const AGENTFLOW_DERIVED_CHANGE_AREAS = new Set(["state"]);

type NextStepViewModel = {
  action: string;
  description: string;
  reason: string;
  status: StatusChipStatus;
  title: string;
};

const pages: Array<{ icon: LucideIcon; id: AppPage; label: string }> = [
  { icon: LayoutDashboard, id: "home", label: "工作台" },
  { icon: ClipboardList, id: "tasks", label: "任务" },
  { icon: FileSearch, id: "files", label: "文件" },
  { icon: ClipboardCheck, id: "delivery", label: "交付" },
  { icon: ShieldCheck, id: "audit", label: "审计" },
  { icon: Settings, id: "advanced", label: "高级" },
];

const onboardingSteps = ["选择项目", "环境准备", "认识 Agent", "确认意图", "完成引导"] as const;

const interactionStorageKeys = {
  activePage: "agentflow.interaction.activePage.v1",
  handedOffIssues: "agentflow.interaction.handedOffIssues.v1",
  onboardingComplete: "agentflow.interaction.onboardingComplete.v1",
  projectRoot: "agentflow.interaction.projectRoot.v1",
  provider: "agentflow.interaction.provider.v1",
} as const;
const appearanceThemeClass = "af-theme-light";
const INCOMPLETE_HANDOFF_MESSAGE = "这个任务包不完整，缺少执行目标。请先修复 Issue 元数据。";

type CodexRoleGuide = {
  cannotDo: string[];
  englishName: string;
  role: AgentRole;
  startupInstruction: string;
  summary: string;
  threadName: string;
  title: string;
};

const codexRoleGuides: CodexRoleGuide[] = [
  {
    cannotDo: ["不改代码", "不执行命令", "不生成 release", "不写 audit report"],
    englishName: "Spec Agent",
    role: "spec-agent",
    startupInstruction: [
      "你现在是 AgentFlow 的 Spec Agent。",
      "",
      "你只做三件事：",
      "1. 确认用户需求。",
      "2. 整理 SPEC。",
      "3. 生成 Issue。",
      "",
      "你不能做：",
      "- 不改代码",
      "- 不执行命令",
      "- 不生成 release",
      "- 不写 audit report",
      "- 不执行 Build Agent 或 Audit Agent 的任务",
      "",
      "你必须遵守：",
      "- 只写 .agentflow/input/**",
      "- 不修改用户源码",
      "- 不写 .agentflow/execute/**",
      "- 不写 .agentflow/output/release/**",
      "- 不写 .agentflow/output/audit/**",
      "",
      "如果用户要求你改代码、执行任务或审计，请停止并提示需要切换到正确 Agent。",
    ].join("\n"),
    summary: "确认需求 · 整理规格 · 生成任务",
    threadName: "AgentFlow / Spec Agent",
    title: "需求助手",
  },
  {
    cannotDo: ["不执行 audit issue", "不写 audit report", "不写 findings.json", "不使用外部任务源"],
    englishName: "Build Agent",
    role: "build-agent",
    startupInstruction: [
      "你现在是 AgentFlow 的 Build Agent。",
      "",
      "你只能执行：",
      "issueCategory = spec",
      "requiredAgentRole = build-agent",
      "AgentFlow 当前 issue / handoff package / executionPipeline 是唯一任务源。",
      "不要把外部 issue、任务、计划、队列、线程或工具状态当成任务源。",
      "不要用外部状态拆分、重排或推进 AgentFlow 任务。",
      "GitHub 命令只允许用于当前 executionPipeline 里的 PR 阶段。",
      "",
      "你要做：",
      "1. GitHub 自动化预检。",
      "2. Agent 执行 issue。",
      "3. 沙箱测试与结果验证。",
      "4. 创建 PR。",
      "5. 合并 PR。",
      "6. 写回 Done。",
      "",
      "你不能做：",
      "- 不执行 audit issue",
      "- 不写 audit report",
      "- 不写 findings.json",
      "- 不写 evidence-map.json",
      "- 不写 traceability.json",
      "- 不越过任务边界",
      "- 不绕过 GitHub 自动化预检",
      "- 不绕过沙箱验证",
      "- 不越过 mergeMode 合并 PR",
      "- 不把外部 issue / task / plan / queue 当成任务源",
      "- 不用外部状态拆分、重排或推进 AgentFlow 任务",
      "- 不 deploy",
      "",
      "创建 PR 前必须完成 GitHub 自动化预检和沙箱验证。",
      "合并 PR 只能按 mergeMode：manual-merge 或 auto-merge-if-eligible。",
      "如果 mergeMode = auto-merge-if-eligible，不能停在 Draft PR；必须执行 gh pr ready、gh pr merge --auto，并轮询 PR 是否 merged。",
      "如果 mergeMode = manual-merge，PR ready 后等待人合并，合并前不能写回 Done。",
      "写回 Done 前必须确认当前 AgentFlow CLI 支持 build-agent complete。",
      "如果使用 target/release/agentflow，必须先运行 cargo build --release --bin agentflow；否则使用 target/debug/agentflow。",
      "不要直接复用可能过期的 target/release/agentflow。",
      "如果任务不是 spec issue，必须停止。",
      "如果 requiredAgentRole 不是 build-agent，必须停止。",
    ].join("\n"),
    summary: "任务打包 · 执行改动 · 写回结果",
    threadName: "AgentFlow / Build Agent",
    title: "执行助手",
  },
  {
    cannotDo: ["不改代码", "不执行 spec issue", "不生成 release", "不创建 PR / merge / deploy"],
    englishName: "Audit Agent",
    role: "audit-agent",
    startupInstruction: [
      "你现在是 AgentFlow 的 Audit Agent。",
      "",
      "你只能执行：",
      "issueCategory = audit",
      "requiredAgentRole = audit-agent",
      "",
      "你要做：",
      "1. 读取 Audit Issue。",
      "2. 读取关联 SPEC / Issue / Evidence / Release。",
      "3. 检查是否符合需求、范围和边界。",
      "4. 写入 audit report。",
      "5. 写入 findings.json。",
      "6. 写入 evidence-map.json。",
      "7. 写入 traceability.json。",
      "",
      "你不能做：",
      "- 不改代码",
      "- 不执行 spec issue",
      "- 不生成 release",
      "- 不创建 PR",
      "- 不 merge",
      "- 不 deploy",
      "- 不修改用户源码",
      "",
      "如果任务不是 audit issue，必须停止。",
      "如果 requiredAgentRole 不是 audit-agent，必须停止。",
    ].join("\n"),
    summary: "审计交付 · 核对证据 · 生成报告",
    threadName: "AgentFlow / Audit Agent",
    title: "审计助手",
  },
];

function readStoredProvider(): Provider | null {
  const value = window.localStorage.getItem(interactionStorageKeys.provider);
  return value === "ChatGPT" || value === "Claude" || value === "DeepSeek" ? value : null;
}

function readStoredPage(): AppPage {
  const value = window.localStorage.getItem(interactionStorageKeys.activePage);
  return isAgentFlowProjectPage(value) ? value : "home";
}

function readStoredProjectRoot() {
  return window.localStorage.getItem(interactionStorageKeys.projectRoot);
}

function readInitialProjectRegistry() {
  if (isBrowserPreviewRuntime() && window.localStorage.getItem(projectRegistryStorageKeys.projects) === null) {
    return createBrowserPreviewProjectRegistry(BROWSER_PREVIEW_PROJECT_ROOT);
  }

  return readProjectRegistry({
    legacyActivePage: readStoredPage(),
    legacyProjectRoot: readStoredProjectRoot(),
    projectNameFromRoot: (root) => projectNameFromPath(root) || "本地项目",
  });
}

function readStoredBoolean(key: string) {
  return window.localStorage.getItem(key) === "true";
}

function readStoredIssueSet() {
  try {
    const value = JSON.parse(window.localStorage.getItem(interactionStorageKeys.handedOffIssues) ?? "[]");
    return new Set(Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : []);
  } catch {
    return new Set<string>();
  }
}

function startWindowDrag(event: MouseEvent<HTMLElement>) {
  if (isBrowserPreviewRuntime() || event.button !== 0) {
    return;
  }

  const target = event.target;
  if (
    target instanceof HTMLElement &&
    target.closest("button, a, input, textarea, select, [data-agentflow-no-drag]")
  ) {
    return;
  }

  void getCurrentWindow().startDragging().catch(() => undefined);
}

function App() {
  const [connectedProvider, setConnectedProvider] = useState<Provider>(() => readStoredProvider() ?? "ChatGPT");
  const [onboardingComplete, setOnboardingComplete] = useState(true);
  const [firstRunOpen, setFirstRunOpen] = useState(false);
  const [projectRegistry, setProjectRegistry] = useState(readInitialProjectRegistry);
  const [taskSearch, setTaskSearch] = useState("");
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [selectedTaskProjectId, setSelectedTaskProjectId] = useState<string | null>(null);
  const [selectedDeliveryRunId, setSelectedDeliveryRunId] = useState<string | null>(null);
  const [selectedAuditId, setSelectedAuditId] = useState<string | null>(null);
  const [inputStatusRefreshToken, setInputStatusRefreshToken] = useState(0);
  const [taskListRefreshToken, setTaskListRefreshToken] = useState(0);
  const [executeRefreshToken, setExecuteRefreshToken] = useState(0);
  const [outputRefreshToken, setOutputRefreshToken] = useState(0);
  const [stateRefreshToken, setStateRefreshToken] = useState(0);
  const [selectedIntent, setSelectedIntent] = useState("我要新增功能");
  const [onboardingFeedback, setOnboardingFeedback] = useState<string | null>(null);
  const [taskActionFeedback, setTaskActionFeedback] = useState<string | null>(null);
  const [taskCopyState, setTaskCopyState] = useState<ButtonInteractionState>("enabled");
  const [handedOffIssues, setHandedOffIssues] = useState<Set<string>>(() => readStoredIssueSet());
  const preparedProjectRoots = useRef(new Set<string>());
  const { activePageByProject, activeProjectRoot, expandedProjectRoots, projects } = projectRegistry;
  const activeProject = projects.find((project) => project.root === activeProjectRoot) ?? null;
  const activeProjectRegistryStatus = activeProject?.status ?? null;
  const projectRoot = activeProject?.root ?? null;
  const activePage = projectRoot ? activePageByProject[projectRoot] ?? activeProject?.lastActivePage ?? "home" : "home";

  const {
    loadProjectDirectoryPage,
    loadProjectFileTextRange,
    loadProjectFiles,
    projectFilesState,
    reportProjectFilesError,
    searchProjectFiles,
    selectProjectFile,
    setProjectFileViewMode,
  } = useProjectFiles(projectRoot);
  const { agentManualState, loadAgentManual } = useAgentManual(projectRoot);
  const { projectPanelState, prepareProjectPanel } = useProjectPanel(projectRoot);
  const inputStatusState = useInputStatus(projectRoot, inputStatusRefreshToken);
  const inputSnapshotState = useInputSnapshot(projectRoot, taskListRefreshToken);
  const executeStatusState = useExecuteStatus(projectRoot, executeRefreshToken);
  const outputStatusState = useOutputStatus(projectRoot, outputRefreshToken);
  const stateStatusState = useStateStatus(projectRoot, stateRefreshToken);
  const issueStatusIndexState = useIssueStatusIndex(
    projectRoot,
    taskListRefreshToken + executeRefreshToken + outputRefreshToken,
  );
  const workspaceData = useWorkspaceData(projectRoot);
  const outputBundle = useOutputBundle(projectRoot, outputRefreshToken);
  const initializationState = useProjectInitializationStatus(projectRoot, outputRefreshToken);

  useEffect(() => {
    if (connectedProvider) {
      window.localStorage.setItem(interactionStorageKeys.provider, connectedProvider);
    }
  }, [connectedProvider]);

  useEffect(() => {
    window.localStorage.setItem(interactionStorageKeys.onboardingComplete, String(onboardingComplete));
  }, [onboardingComplete]);

  useEffect(() => {
    if (projectRoot) {
      window.localStorage.setItem(interactionStorageKeys.projectRoot, projectRoot);
    } else {
      window.localStorage.removeItem(interactionStorageKeys.projectRoot);
    }
  }, [projectRoot]);

  useEffect(() => {
    persistProjectRegistry(projectRegistry);
  }, [projectRegistry]);

  useEffect(() => {
    window.localStorage.setItem(interactionStorageKeys.activePage, activePage);
  }, [activePage]);

  useEffect(() => {
    window.localStorage.setItem(interactionStorageKeys.handedOffIssues, JSON.stringify([...handedOffIssues]));
  }, [handedOffIssues]);

  useEffect(() => {
    if (projectRoot) {
      return;
    }
    setSelectedTaskId(null);
    setSelectedDeliveryRunId(null);
    setSelectedAuditId(null);
    setTaskSearch("");
    setTaskActionFeedback(null);
  }, [projectRoot]);

  useEffect(() => {
    if (!projectRoot) {
      return;
    }

    void loadProjectFiles(projectRoot);
    void loadAgentManual(projectRoot);
  }, [projectRoot]);

  useEffect(() => {
    if (!projectRoot || isBrowserPreviewRuntime()) {
      return;
    }

    let cancelled = false;
    let unlisten: (() => void) | null = null;
    let startAttempt = 0;
    let refreshTimer: number | null = null;
    let lastRefreshAt = 0;
    const queuedAreas = new Set<string>();

    const flushQueuedRefresh = () => {
      refreshTimer = null;
      if (cancelled) {
        return;
      }

      const changedAreas = new Set(queuedAreas);
      queuedAreas.clear();
      if (changedAreas.size === 0) {
        return;
      }

      lastRefreshAt = Date.now();
      const refreshAll = changedAreas.has("all");
      const refreshInput = refreshAll || changedAreas.has("input");
      const refreshExecute = refreshAll || changedAreas.has("execute");
      const refreshOutput = refreshAll || changedAreas.has("output");
      const refreshPanel = refreshAll || changedAreas.has("panel");
      const refreshTaskList = refreshInput || refreshExecute || refreshOutput || refreshAll;

      if (refreshTaskList) {
        setTaskListRefreshToken((current) => current + 1);
      }

      if (refreshOutput && activePage === "delivery") {
        setOutputRefreshToken((current) => current + 1);
      }
      if (refreshOutput && activePage === "audit") {
        setOutputRefreshToken((current) => current + 1);
      }
      if (refreshPanel && activePage === "files") {
        void loadProjectFiles(projectRoot);
      }
    };

    const scheduleQueuedRefresh = () => {
      if (refreshTimer !== null) {
        return;
      }
      const elapsed = Date.now() - lastRefreshAt;
      const delay = Math.max(
        AGENTFLOW_WATCHER_REFRESH_DELAY_MS,
        AGENTFLOW_WATCHER_REFRESH_COOLDOWN_MS - elapsed,
      );
      refreshTimer = window.setTimeout(flushQueuedRefresh, delay);
    };

    const startWatcher = () => {
      startAttempt += 1;
      void invoke("start_agentflow_workspace_watcher", { projectRoot }).catch(() => {
        if (!cancelled && startAttempt < 5) {
          window.setTimeout(startWatcher, 800);
        }
      });
    };

    void listen<AgentflowWorkspaceChangedEvent>(AGENTFLOW_WORKSPACE_CHANGED_EVENT, (event) => {
      if (cancelled) {
        return;
      }
      const payload = event.payload;
      if (normalizeProjectRootKey(payload.projectRoot) !== normalizeProjectRootKey(projectRoot)) {
        return;
      }
      const changedAreas = payload.changedAreas.filter(
        (area) => area === "all" || !AGENTFLOW_DERIVED_CHANGE_AREAS.has(area),
      );
      if (changedAreas.length === 0) {
        return;
      }
      changedAreas.forEach((area) => queuedAreas.add(area));
      scheduleQueuedRefresh();
    }).then((cleanup) => {
      if (cancelled) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    startWatcher();

    return () => {
      cancelled = true;
      if (refreshTimer !== null) {
        window.clearTimeout(refreshTimer);
      }
      unlisten?.();
    };
  }, [activePage, loadProjectFiles, projectRoot]);

  useEffect(() => {
    if (!projectRoot || isBrowserPreviewRuntime() || preparedProjectRoots.current.has(projectRoot)) {
      return;
    }

    let cancelled = false;
    preparedProjectRoots.current.add(projectRoot);
    setProjectRegistry((current) =>
      upsertProject(current, {
        ...createProjectRef({
          expanded: true,
          lastActivePage: activePage,
          name: projectNameFromPath(projectRoot) || "本地项目",
          root: projectRoot,
          status: "loading",
        }),
      }),
    );

    void invoke<ProjectWorkspaceSummary>("prepare_local_project_workspace", {
      appLocale: detectAppLocale(),
      projectRoot,
    })
      .then((summary) => {
        if (cancelled) {
          return;
        }
        setProjectRegistry((current) =>
          upsertProject(current, {
            ...createProjectRef({
              expanded: true,
              lastActivePage: activePage,
              name: projectNameFromPath(projectRoot) || "本地项目",
              root: projectRoot,
              status: "ready",
            }),
          }),
        );
        setInputStatusRefreshToken((current) => current + 1);
        setTaskListRefreshToken((current) => current + 1);
        setExecuteRefreshToken((current) => current + 1);
        setOutputRefreshToken((current) => current + 1);
        setStateRefreshToken((current) => current + 1);
        void loadProjectFiles(projectRoot);
        void loadAgentManual(projectRoot);
        setOnboardingFeedback(summary.initializationStatus?.message ?? null);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        preparedProjectRoots.current.delete(projectRoot);
        if (cancelled) {
          return;
        }
        setProjectRegistry((current) =>
          upsertProject(current, {
            ...createProjectRef({
              expanded: true,
              lastActivePage: activePage,
              name: projectNameFromPath(projectRoot) || "本地项目",
              root: projectRoot,
              status: "error",
            }),
            error: message,
          }),
        );
        reportProjectFilesError(message);
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot]);

  const tasks = useMemo(
    () =>
      buildTaskItems(
        inputSnapshotState.snapshot ? inputSnapshotState.snapshot.issues : null,
        issueStatusIndexState.index,
        workspaceData.projectViewModel,
        workspaceData.workbench,
      ),
    [inputSnapshotState.snapshot, issueStatusIndexState.index, workspaceData.projectViewModel, workspaceData.workbench],
  );
  const contextPackDispatchKey = useMemo(
    () =>
      (inputSnapshotState.snapshot?.issues ?? [])
        .map((issue) =>
          [
            issue.issueId,
            issue.displayStatus,
            issue.status,
            issue.contextPackPath ?? "",
            issue.system?.revision ?? 0,
          ].join(":"),
        )
        .join("|"),
    [inputSnapshotState.snapshot],
  );

  useEffect(() => {
    if (
      !projectRoot ||
      isBrowserPreviewRuntime() ||
      inputSnapshotState.source !== "tauri" ||
      !inputSnapshotState.snapshot?.issues.length
    ) {
      return;
    }

    let cancelled = false;
    void invoke<WorkflowEventDispatchSummary>("dispatch_workflow_events", { projectRoot })
      .then((summary) => {
        if (cancelled) {
          return;
        }
        if (summary.contextPackReady > 0 || summary.contextPackFailed > 0) {
          void prepareProjectPanel(projectRoot);
          setStateRefreshToken((current) => current + 1);
        }
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
    };
  }, [
    contextPackDispatchKey,
    inputSnapshotState.snapshot?.issues.length,
    inputSnapshotState.source,
    prepareProjectPanel,
    projectRoot,
  ]);

  const filteredTasks = useMemo(() => {
    const query = taskSearch.trim().toLowerCase();
    if (!query) {
      return tasks;
    }
    return tasks.filter((task) => {
      const searchable = [task.id, task.title, task.displayStatus, task.status, task.riskLevel, task.goal]
        .join(" ")
        .toLowerCase();
      return searchable.includes(query);
    });
  }, [taskSearch, tasks]);
  const activeIssueId = workspaceData.workbench?.goalLoop?.activeIssueId ?? null;
  const taskProjectTree = useMemo(
    () =>
      inputSnapshotState.snapshot
        ? buildTaskProjectTreeViewModel({
            activeIssueId,
            issueStatusIndex: issueStatusIndexState.index,
            issues: inputSnapshotState.snapshot.issues,
            projects: inputSnapshotState.snapshot.projects,
            relations: inputSnapshotState.snapshot.relations,
          })
        : null,
    [activeIssueId, inputSnapshotState.snapshot, issueStatusIndexState.index],
  );
  const selectedProjectGroup = useMemo(
    () => taskProjectTree?.groups.find((group) => group.id === selectedTaskProjectId) ?? null,
    [selectedTaskProjectId, taskProjectTree],
  );
  const selectedTaskCandidateId = useMemo(
    () => (selectedProjectGroup ? null : pickTaskId(tasks, selectedTaskId, activeIssueId)),
    [activeIssueId, selectedProjectGroup, selectedTaskId, tasks],
  );
  const taskInteractionState = useMemo(
    () => buildTaskInteractionState(tasks, selectedTaskCandidateId),
    [selectedTaskCandidateId, tasks],
  );
  const selectedTask = taskInteractionState.selectedTask;
  const nextStep = useMemo(
    () => buildNextStep(stateStatusState, inputStatusState.status, outputStatusState, selectedTask),
    [inputStatusState.status, outputStatusState, selectedTask, stateStatusState],
  );
  const activeProjectLiveStatus = projectRoot
    ? activeProjectRegistryStatus === "missing"
      ? "missing"
      : activeProjectStatus(projectFilesState, stateStatusState)
    : null;
  const appInteractionState: AppInteractionState = useMemo(
    () => {
      const outputPageHasError = (activePage === "delivery" || activePage === "audit") && Boolean(outputBundle.error);
      return buildAppInteractionState({
        activePage,
        hasError: Boolean(
          outputPageHasError || activeProjectLiveStatus === "error" || activeProjectLiveStatus === "missing",
        ),
        onboardingComplete,
        projectLoading:
          activeProjectLiveStatus === "loading" || projectFilesState.loading || workspaceData.source === "loading",
        projectRoot,
        providerConnected: Boolean(connectedProvider),
        workspaceBlocked: Boolean(stateStatusState.status?.blockers.length),
      });
    },
    [
      activePage,
      activeProjectLiveStatus,
      connectedProvider,
      onboardingComplete,
      outputBundle.error,
      projectFilesState.loading,
      projectRoot,
      stateStatusState.status?.blockers.length,
      workspaceData.source,
    ],
  );

  useEffect(() => {
    if (selectedTaskProjectId) {
      if (!taskProjectTree?.groups.some((group) => group.id === selectedTaskProjectId)) {
        setSelectedTaskProjectId(null);
      }
      return;
    }

    const nextTaskId = pickTaskId(tasks, selectedTaskId, activeIssueId);
    if (nextTaskId !== selectedTaskId) {
      setSelectedTaskId(nextTaskId);
    }
  }, [activeIssueId, selectedTaskId, selectedTaskProjectId, taskProjectTree, tasks]);

  function refreshProjectPage(page: AppPage, root = projectRoot) {
    if (!root) {
      return;
    }

    if (page === "home") {
      void prepareProjectPanel(root);
      setInputStatusRefreshToken((current) => current + 1);
      setTaskListRefreshToken((current) => current + 1);
      setOutputRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
      return;
    }

    if (page === "tasks") {
      setTaskListRefreshToken((current) => current + 1);
      return;
    }

    if (page === "files") {
      void loadProjectFiles(root);
      return;
    }

    if (page === "delivery" || page === "audit") {
      setOutputRefreshToken((current) => current + 1);
      return;
    }

    if (page === "advanced") {
      void loadProjectFiles(root);
      void loadAgentManual(root);
      void prepareProjectPanel(root);
      setInputStatusRefreshToken((current) => current + 1);
      setTaskListRefreshToken((current) => current + 1);
      setExecuteRefreshToken((current) => current + 1);
      setOutputRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
    }
  }

  function setActivePage(page: AppPage) {
    const shouldRefresh = page !== activePage;
    setProjectRegistry((current) => setProjectPage(current, current.activeProjectRoot, page));
    if (shouldRefresh) {
      refreshProjectPage(page);
    }
  }

  function handleSelectProject(projectRootToSelect: string) {
    setProjectRegistry((current) => selectProject(current, projectRootToSelect));
    setTaskSearch("");
    setSelectedTaskProjectId(null);
    setOutputRefreshToken((current) => current + 1);
  }

  function handleToggleProject(projectRootToToggle: string) {
    setProjectRegistry((current) => toggleProjectExpanded(current, projectRootToToggle));
  }

  function handleRemoveProject(projectRootToRemove: string) {
    const removingActiveProject = projectRootToRemove === projectRoot;
    setProjectRegistry((current) => removeProject(current, projectRootToRemove));
    if (removingActiveProject) {
      setSelectedTaskId(null);
      setSelectedTaskProjectId(null);
      setSelectedDeliveryRunId(null);
      setSelectedAuditId(null);
      setTaskSearch("");
      setTaskActionFeedback(null);
      setOutputRefreshToken((current) => current + 1);
    }
  }

  function handleProjectPageChange(projectRootToSelect: string, page: AppPage) {
    const shouldRefresh = projectRootToSelect !== projectRoot || page !== activePage;
    setProjectRegistry((current) => setProjectPage(current, projectRootToSelect, page));
    setTaskSearch("");
    if (shouldRefresh) {
      refreshProjectPage(page, projectRootToSelect);
    }
  }

  function handleSelectTask(taskId: string) {
    setSelectedTaskProjectId(null);
    setSelectedTaskId(taskId);
    setTaskActionFeedback(null);
  }

  function handleSelectTaskProject(projectId: string) {
    setSelectedTaskId(null);
    setSelectedTaskProjectId(projectId);
    setTaskActionFeedback(null);
  }

  async function chooseProjectFolder() {
    if (isBrowserPreviewRuntime()) {
      setProjectRegistry((current) =>
        upsertProject(
          current,
          createProjectRef({
            expanded: true,
            lastActivePage: "home",
            name: projectNameFromPath(BROWSER_PREVIEW_PROJECT_ROOT) || "AgentFlow",
            root: BROWSER_PREVIEW_PROJECT_ROOT,
            status: "ready",
          }),
        ),
      );
      setOnboardingFeedback("浏览器预览使用 mock 项目现场，不会读取或写入真实本地项目。");
      return;
    }

    let normalizedRoot: string | null = null;
    try {
      const selectedRoot = await invoke<string | null>("choose_existing_project_folder");
      normalizedRoot = selectedRoot ? normalizeProjectRootKey(selectedRoot) : null;
      if (!normalizedRoot) {
        return;
      }

      const projectRootToAdd = normalizedRoot;
      const projectName = projectNameFromPath(projectRootToAdd) || "本地项目";
      setOnboardingFeedback("正在准备项目工作规则和现场。");
      setProjectRegistry((current) =>
        upsertProject(
          current,
          createProjectRef({
            expanded: true,
            lastActivePage: "home",
            name: projectName,
            root: projectRootToAdd,
            status: "loading",
          }),
        ),
      );
      const summary = await invoke<ProjectWorkspaceSummary>("prepare_local_project_workspace", {
        appLocale: detectAppLocale(),
        projectRoot: projectRootToAdd,
      });
      setProjectRegistry((current) =>
        upsertProject(
          current,
          createProjectRef({
            expanded: true,
            lastActivePage: "home",
            name: projectName,
            root: projectRootToAdd,
            status: "ready",
          }),
        ),
      );
      setOutputRefreshToken((current) => current + 1);
      setOnboardingFeedback(summary.initializationStatus?.message ?? "项目已准备好。");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setOnboardingFeedback(message);
      if (normalizedRoot) {
        const failedProjectRoot = normalizedRoot;
        setProjectRegistry((current) =>
          upsertProject(
            current,
            {
              ...createProjectRef({
                expanded: true,
                lastActivePage: "home",
                name: projectNameFromPath(failedProjectRoot) || "本地项目",
                root: failedProjectRoot,
                status: "error",
              }),
              error: message,
            },
          ),
        );
      }
      reportProjectFilesError(message);
    }
  }

  function refreshWorkspace() {
    if (!projectRoot) {
      return;
    }
    void loadProjectFiles(projectRoot);
    void loadAgentManual(projectRoot);
    void prepareProjectPanel(projectRoot);
    setInputStatusRefreshToken((current) => current + 1);
    setTaskListRefreshToken((current) => current + 1);
    setExecuteRefreshToken((current) => current + 1);
    setOutputRefreshToken((current) => current + 1);
    setStateRefreshToken((current) => current + 1);
  }

  async function handleTaskAction(action: TaskInteractionAction, task: V1Issue) {
    setTaskActionFeedback(null);
    if (action === "copy-handoff") {
      const validationError = taskHandoffValidationError(task);
      if (validationError) {
        setTaskCopyState("error");
        setTaskActionFeedback(validationError);
        window.setTimeout(() => setTaskCopyState("enabled"), 1800);
        return;
      }
      setTaskCopyState("loading");
      try {
        await navigator.clipboard.writeText(buildCodexHandoff(task));
        setTaskCopyState("success");
        setTaskActionFeedback("已复制，请粘贴到 Build Agent 会话中。");
        window.setTimeout(() => setTaskCopyState("enabled"), 1400);
      } catch {
        setTaskCopyState("error");
        setTaskActionFeedback("复制失败。请手动复制 Agent Handoff Package。");
      }
      return;
    }

    if (action === "mark-handed-off") {
      setHandedOffIssues((current) => new Set(current).add(task.id));
      setTaskActionFeedback("已做本地标记。AgentFlow 不会自动控制执行过程。");
      return;
    }

    if (action === "check-writeback") {
      refreshWorkspace();
      const delivery = findDeliveryForTask(outputBundle.outputIndex?.releaseDeliveries ?? [], task.id);
      if (delivery) {
        setSelectedDeliveryRunId(delivery.runId);
        setActivePage("delivery");
      } else {
        setTaskActionFeedback("还没有检测到写回结果。");
      }
      return;
    }

    if (action === "view-delivery") {
      const delivery = findDeliveryForTask(outputBundle.outputIndex?.releaseDeliveries ?? [], task.id);
      if (delivery) {
        setSelectedDeliveryRunId(delivery.runId);
        setActivePage("delivery");
      } else {
        setTaskActionFeedback("还没有交付结果。写回后会显示在交付页。");
      }
      return;
    }

    if (action === "view-requirement") {
      setTaskActionFeedback("需求详情来自已确认规格。普通页面暂不展示原始规格。");
      return;
    }

    setTaskActionFeedback("这个任务当前只读查看。");
  }

  function handleLogin(provider: Provider) {
    setConnectedProvider(provider);
    setFirstRunOpen(!onboardingComplete);
  }

  function completeOnboarding() {
    setOnboardingComplete(true);
    setFirstRunOpen(false);
    setActivePage("home");
    refreshWorkspace();
  }

  const projectDisplayName = projectNameFromPath(projectRoot ?? "") || "未选择项目";
  const projectAvailabilityStatus =
    activeProjectLiveStatus === "loading" ||
    activeProjectLiveStatus === "error" ||
    activeProjectLiveStatus === "missing"
      ? activeProjectLiveStatus
      : null;
  const navigationProjects = projectsWithLiveStatus(projects, projectRoot, projectFilesState, stateStatusState);
  const activeNavigationProject = navigationProjects.find((project) => project.root === projectRoot) ?? null;
  const toolbar = projectRoot ? (
    <Toolbar
      activePage={activePage}
      onRefresh={refreshWorkspace}
      onSearchChange={setTaskSearch}
      taskSearch={taskSearch}
    />
  ) : null;
  const titlebarProjectName = projectRoot ? projectDisplayName : "AgentFlow";
  const titlebarStatus = projectRoot
    ? titlebarStatusText(appInteractionState, stateStatusState.status?.currentStage, selectedTask)
    : "未选择项目 · 本地模式";

  return (
    <>
      <AppShell
        activePage={activePage}
        activeProjectRoot={activeProjectRoot}
        expandedProjectRoots={expandedProjectRoots}
        inspector={null}
        onAddProject={chooseProjectFolder}
        onPageChange={handleProjectPageChange}
        onRemoveProject={handleRemoveProject}
        onSelectProject={handleSelectProject}
        onToggleProject={handleToggleProject}
        projectName={titlebarProjectName}
        projectRoot={projectRoot}
        projects={navigationProjects}
        statusBar={
          <StatusBar
            projectName={projectDisplayName}
            projectRoot={projectRoot}
            projectStatus={activeNavigationProject?.status ?? null}
            appInteractionState={appInteractionState}
            stateStatus={stateStatusState.status}
          />
        }
        titlebarStatus={titlebarStatus}
        toolbar={toolbar}
      >
        {!projectRoot ? <EmptyProjectPage onAddProject={chooseProjectFolder} /> : null}
        {projectRoot && projectAvailabilityStatus ? (
          <ProjectAvailabilityPage
            error={activeProject?.error}
            onAddProject={chooseProjectFolder}
            projectName={projectDisplayName}
            status={projectAvailabilityStatus}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "home" ? (
          <ProjectHomePage
            connectedProvider={connectedProvider}
            nextStep={nextStep}
            onOpenAudit={() => setActivePage("audit")}
            onOpenDelivery={() => setActivePage("delivery")}
            onOpenFiles={() => setActivePage("files")}
            onOpenTasks={() => setActivePage("tasks")}
            outputBundle={outputBundle}
            outputStatusState={outputStatusState}
            projectPanelState={projectPanelState}
            projectFilesState={projectFilesState}
            projectName={projectDisplayName}
            projectRoot={projectRoot}
            selectedTask={selectedTask}
            initializationState={initializationState}
            stateStatusState={stateStatusState}
            workspaceData={workspaceData}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "tasks" ? (
          <TasksPage
            actionFeedback={taskActionFeedback}
            actions={taskInteractionState.actions}
            copyState={taskCopyState}
            onTaskAction={(action, task) => void handleTaskAction(action, task)}
            onSelectProjectGroup={handleSelectTaskProject}
            onSelectTask={handleSelectTask}
            projectRoot={projectRoot}
            selectedProjectGroup={selectedProjectGroup}
            selectedTask={selectedTask}
            suggestions={initializationState.status?.recentContext ?? []}
            taskTree={taskProjectTree}
            tasks={filteredTasks}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "files" ? (
          <FilesPage
            fileState={projectFilesState}
            onChangeViewMode={setProjectFileViewMode}
            onLoadDirectoryPage={loadProjectDirectoryPage}
            onLoadTextRange={loadProjectFileTextRange}
            onSearchFiles={searchProjectFiles}
            onSelectFile={selectProjectFile}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "delivery" ? (
          <DeliveryPage
            onOpenAudit={() => setActivePage("audit")}
            onSelectDelivery={setSelectedDeliveryRunId}
            outputBundle={outputBundle}
            outputStatusState={outputStatusState}
            selectedDeliveryRunId={selectedDeliveryRunId}
            selectedTask={selectedTask}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "audit" ? (
          <AuditPage
            onSelectAudit={setSelectedAuditId}
            outputBundle={outputBundle}
            selectedAuditId={selectedAuditId}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "advanced" ? (
          <AdvancedPage
            agentManualState={agentManualState}
            executeStatusState={executeStatusState}
            inputStatusState={inputStatusState}
            outputBundle={outputBundle}
            outputStatusState={outputStatusState}
            projectFilesState={projectFilesState}
            projectPanelState={projectPanelState}
            initializationState={initializationState}
            stateStatusState={stateStatusState}
            workspaceData={workspaceData}
          />
        ) : null}
      </AppShell>

      {firstRunOpen ? (
        <FirstRunModal
          feedback={onboardingFeedback}
          onChooseProject={() => void chooseProjectFolder()}
          onClose={completeOnboarding}
          onIntentChange={setSelectedIntent}
          projectRoot={projectRoot}
          selectedIntent={selectedIntent}
        />
      ) : null}
    </>
  );
}

function useWorkspaceData(projectRoot: string | null): WorkspaceDataState {
  const [state, setState] = useState<WorkspaceDataState>({
    error: null,
    projectViewModel: null,
    source: "idle",
    workbench: null,
  });

  useEffect(() => {
    if (!projectRoot) {
      setState({ error: null, projectViewModel: null, source: "idle", workbench: null });
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setState({
        error: null,
        projectViewModel: createBrowserPreviewProjectViewModelSnapshot(projectRoot),
        source: "preview",
        workbench: createBrowserPreviewWorkbenchSnapshot(projectRoot),
      });
      return;
    }

    let cancelled = false;
    setState((current) => ({ ...current, error: null, source: "loading" }));
    void Promise.all([
      invoke<WorkbenchSnapshot>("load_workbench_snapshot"),
      invoke<ProjectMilestoneIssueViewModelSnapshot>("load_project_milestone_issue_view_model_snapshot"),
    ])
      .then(([workbench, projectViewModel]) => {
        if (!cancelled) {
          setState({ error: null, projectViewModel, source: "tauri", workbench });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setState({
            error: error instanceof Error ? error.message : String(error),
            projectViewModel: null,
            source: "unavailable",
            workbench: null,
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot]);

  return state;
}

function useOutputBundle(projectRoot: string | null, refreshToken: number): OutputBundleState {
  const [state, setState] = useState<OutputBundleState>({
    auditIndex: null,
    auditReport: null,
    error: null,
    outputIndex: null,
    source: "idle",
  });

  useEffect(() => {
    if (!projectRoot) {
      setState({ auditIndex: null, auditReport: null, error: null, outputIndex: null, source: "idle" });
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setState({
        auditIndex: createBrowserPreviewAuditIndex(),
        auditReport: createBrowserPreviewHumanAuditReport(),
        error: null,
        outputIndex: createBrowserPreviewOutputIndex(),
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setState((current) =>
      current.outputIndex || current.auditIndex
        ? { ...current, error: null }
        : { ...current, error: null, source: "loading" },
    );
    void Promise.all([
      invoke<OutputIndex>("load_output_index", { projectRoot }),
      invoke<AuditIndex>("load_audit_index", { projectRoot }),
    ])
      .then(async ([outputIndex, auditIndex]) => {
        const latestAuditWithReport = sortAuditsByLatest(auditIndex.audits).find((audit) =>
          auditHasReport(audit),
        );
        const auditReport = latestAuditWithReport
          ? await invoke<HumanAuditReport>("load_audit_report", { auditId: latestAuditWithReport.auditId, projectRoot })
          : null;

        if (!cancelled) {
          setState({ auditIndex, auditReport, error: null, outputIndex, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : String(error);
          setState((current) =>
            current.outputIndex || current.auditIndex
              ? { ...current, error: message }
              : {
                  auditIndex: null,
                  auditReport: null,
                  error: message,
                  outputIndex: null,
                  source: "unavailable",
                },
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, refreshToken]);

  return state;
}

function useProjectInitializationStatus(
  projectRoot: string | null,
  refreshToken: number,
): ProjectInitializationState {
  const [state, setState] = useState<ProjectInitializationState>({
    error: null,
    source: "idle",
    status: null,
  });

  useEffect(() => {
    if (!projectRoot) {
      setState({ error: null, source: "idle", status: null });
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setState({
        error: null,
        source: "preview",
        status: {
          demoAuditCount: 1,
          demoDataCreated: true,
          demoDeliveryCount: 1,
          demoIssueCount: 5,
          gitContextLoaded: false,
          initialized: true,
          message: "浏览器预览使用本地 mock 数据。",
          paths: [],
          projectKind: "new",
          recentContext: [],
          recentContextCount: 0,
          version: "base-release-initialization.browser-preview",
          warnings: ["浏览器预览不写 .agentflow。"],
        },
      });
      return;
    }

    let cancelled = false;
    setState((current) => ({ ...current, error: null, source: "loading" }));
    void invoke<ProjectInitializationStatus>("load_project_initialization_status", { projectRoot })
      .then((status) => {
        if (!cancelled) {
          setState({ error: null, source: "tauri", status });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setState({
            error: error instanceof Error ? error.message : String(error),
            source: "unavailable",
            status: null,
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, refreshToken]);

  return state;
}

function LoginModal({ onConnect }: { onConnect: (provider: Provider) => void }) {
  const providers: Array<{ id: Provider }> = [
    { id: "ChatGPT" },
    { id: "Claude" },
    { id: "DeepSeek" },
  ];
  const runtimeChromeClass = isBrowserPreviewRuntime() ? "browser-preview-titlebar" : "native-titlebar";

  return (
    <main
      className={`v16-login-stage v16-login-shell ${runtimeChromeClass} ${appearanceThemeClass}`}
      data-agentflow-screen="login"
    >
      <header
        className="v16-login-titlebar"
        aria-label="登录窗口"
        data-tauri-drag-region
        onMouseDown={startWindowDrag}
      >
        <div className="v16-titlebar-left" data-tauri-drag-region>
          {isBrowserPreviewRuntime() ? <WindowDots /> : null}
          <span className="v16-titlebar-action muted">未登录</span>
        </div>
        <div className="v16-titlebar-project" data-tauri-drag-region>
          <span className="v16-titlebar-status-dot idle" aria-hidden="true" />
          <strong>AgentFlow</strong>
          <small className="v16-titlebar-status-text idle">not-authenticated</small>
        </div>
        <span className="v16-command-key">⌘K</span>
      </header>
      <section className="v16-login-content" aria-label="连接大模型入口">
        <h1>连接大模型入口</h1>
        <p>选择你将用来配合 AgentFlow 的入口。登录是独立模块，不展示项目内容；完成后进入首次引导。</p>
        <div className="v16-provider-list" role="list">
          {providers.map((provider) => (
            <button
              className="v16-provider-entry"
              key={provider.id}
              onClick={() => onConnect(provider.id)}
              type="button"
            >
              <strong>{provider.id}</strong>
              <span>连接</span>
            </button>
          ))}
        </div>
      </section>
    </main>
  );
}

function FirstRunModal({
  feedback,
  onChooseProject,
  onClose,
  onIntentChange,
  projectRoot,
  selectedIntent,
}: {
  feedback: string | null;
  onChooseProject: () => void;
  onClose: () => void;
  onIntentChange: (intent: string) => void;
  projectRoot: string | null;
  selectedIntent: string;
}) {
  const [stepIndex, setStepIndex] = useState(0);
  const stepTitle = onboardingSteps[stepIndex];
  const isFinalStep = stepIndex === onboardingSteps.length - 1;
  const projectReady = Boolean(projectRoot);

  function nextStep() {
    setStepIndex((current) => Math.min(current + 1, onboardingSteps.length - 1));
  }

  return (
    <div className="v16-modal-backdrop" data-agentflow-screen="first-run">
      <WindowChrome className="v16-floating-window v16-first-run-window" aria-label="首次引导">
        <header className="v16-first-run-header">
          <div>
            <p className="v16-kicker">首次引导</p>
            <h2>{stepTitle}</h2>
          </div>
          <ol aria-label="引导进度">
            {onboardingSteps.map((step, index) => (
              <li className={index === stepIndex ? "active" : index < stepIndex ? "done" : ""} key={step}>
                {index + 1}
              </li>
            ))}
          </ol>
        </header>

        {stepTitle === "选择项目" ? (
          <section className="v16-first-run-body">
            <p>打开一个本地项目，AgentFlow 会准备工作规则和项目现场。</p>
            <div className="v16-project-picker">
              <span>{projectRoot ? projectRoot : "还没有选择项目"}</span>
              <ActionButton leftIcon={<FolderOpen size={16} />} onClick={onChooseProject} variant="primary">
                打开本地项目
              </ActionButton>
            </div>
            {feedback ? <p className="v16-feedback">{feedback}</p> : null}
          </section>
        ) : null}

        {stepTitle === "环境准备" ? (
          <section className="v16-first-run-body">
            <div className="v16-env-summary">
              <strong>{projectRoot ? projectNameFromPath(projectRoot) : "未选择项目"}</strong>
              <span>{projectRoot ?? "请先选择本地项目"}</span>
              <div className="v16-progress" aria-label="准备进度">
                <span style={{ width: projectReady ? "100%" : "35%" }} />
              </div>
            </div>
            <ul className="v16-check-list">
              {[
                "检测项目结构",
                "创建 Agent 工作规则",
                "创建 .agentflow 目录结构",
                "读取项目现场",
                "初始化项目状态",
                "验证环境",
              ].map((item) => (
                <li className={projectReady ? "ready" : ""} key={item}>
                  <CheckCircle2 size={15} />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </section>
        ) : null}

        {stepTitle === "认识 Agent" ? (
          <section className="v16-first-run-body v16-agent-brief">
            <AgentBrief className="spec" title="需求助手" value="确认需求 · 整理规格 · 生成任务" />
            <AgentBrief className="build" title="执行助手" value="任务打包 · 执行改动 · 写回结果" />
            <AgentBrief className="audit" title="审计助手" value="审计交付 · 核对证据 · 生成报告" />
          </section>
        ) : null}

        {stepTitle === "确认意图" ? (
          <section className="v16-first-run-body">
            <div className="v16-intent-grid">
              {["我要开发 APP", "我要重构代码", "我要新增功能", "我要修复 BUG", "我要理解项目"].map((intent) => (
                <button
                  className={intent === selectedIntent ? "selected" : ""}
                  key={intent}
                  onClick={() => onIntentChange(intent)}
                  type="button"
                >
                  {intent}
                </button>
              ))}
            </div>
            <CopyableCodeBlock
              content={`请基于当前项目帮我处理：${selectedIntent}\n先确认需求，再生成可执行的任务包。`}
              maxHeight={118}
              title="在聊天会话中启动输入："
            />
          </section>
        ) : null}

        {stepTitle === "完成引导" ? (
          <section className="v16-first-run-body v16-complete-step">
            <CheckCircle2 size={36} />
            <h3>一切准备就绪</h3>
          </section>
        ) : null}

        <footer className="v16-first-run-actions">
          {isFinalStep ? (
            <ActionButton onClick={onClose} size="lg" variant="primary">
              进入工作台
            </ActionButton>
          ) : (
            <>
              <ActionButton disabled={stepIndex === 0} onClick={() => setStepIndex((current) => Math.max(current - 1, 0))}>
                上一步
              </ActionButton>
              <ActionButton disabled={stepTitle === "选择项目" && !projectReady} onClick={nextStep} variant="primary">
                下一步
              </ActionButton>
            </>
          )}
        </footer>
      </WindowChrome>
    </div>
  );
}

function AgentBrief({ className, title, value }: { className?: string; title: string; value: string }) {
  return (
    <article className={className}>
      <strong>{title}</strong>
      <span>{value}</span>
    </article>
  );
}

function AppShell({
  activePage,
  activeProjectRoot,
  children,
  expandedProjectRoots,
  inspector,
  onAddProject,
  onPageChange,
  onRemoveProject,
  onSelectProject,
  onToggleProject,
  projectName,
  projectRoot,
  projects,
  statusBar,
  titlebarStatus,
  toolbar,
}: {
  activePage: AppPage;
  activeProjectRoot: string | null;
  children: ReactNode;
  expandedProjectRoots: Set<string>;
  inspector: ReactNode;
  onAddProject: () => void;
  onPageChange: (projectRoot: string, page: AppPage) => void;
  onRemoveProject: (projectRoot: string) => void;
  onSelectProject: (projectRoot: string) => void;
  onToggleProject: (projectRoot: string) => void;
  projectName: string;
  projectRoot: string | null;
  projects: AgentFlowProjectRef[];
  statusBar: ReactNode;
  titlebarStatus: string;
  toolbar: ReactNode;
}) {
  const runtimeChromeClass = isBrowserPreviewRuntime() ? "browser-preview-titlebar" : "native-titlebar";
  const workspaceClassName = [
    "v16-workspace",
    inspector ? "with-inspector" : null,
    toolbar ? null : "without-toolbar",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <AppFrame className={`v16-app ${runtimeChromeClass} ${appearanceThemeClass}`} data-agentflow-ux="v16">
      <TitleBar projectName={projectName} statusText={titlebarStatus} />
      <ProjectTree
        activePage={activePage}
        activeProjectRoot={activeProjectRoot}
        expandedProjectRoots={expandedProjectRoots}
        onAddProject={onAddProject}
        onPageChange={onPageChange}
        onRemoveProject={onRemoveProject}
        onSelectProject={onSelectProject}
        onToggleProject={onToggleProject}
        projects={projects}
      />
      <section className={workspaceClassName}>
        {toolbar}
        <section className="v16-main-content">{children}</section>
        {inspector}
      </section>
      {statusBar}
    </AppFrame>
  );
}

function TitleBar({
  projectName,
  statusText,
}: {
  projectName: string;
  statusText: string;
}) {
  const statusTone = titlebarStatusDotStatus(statusText);
  return (
    <TopBar className="v16-titlebar" aria-label="应用顶部栏" data-tauri-drag-region onMouseDown={startWindowDrag}>
      <div className="v16-titlebar-left" data-tauri-drag-region>
        {isBrowserPreviewRuntime() ? <WindowDots /> : null}
      </div>
      <div className="v16-titlebar-project" data-tauri-drag-region>
        <span className={`v16-titlebar-status-dot ${statusTone}`} aria-hidden="true" />
        <strong>{projectName}</strong>
        <small className={`v16-titlebar-status-text ${statusTone}`}>{statusText}</small>
      </div>
      <div className="v16-titlebar-right">
        <span className="v16-command-key">⌘K</span>
      </div>
    </TopBar>
  );
}

function WindowDots() {
  return (
    <span className="v16-window-dots" aria-hidden="true">
      <i />
      <i />
      <i />
    </span>
  );
}

function ProjectTree({
  activePage,
  activeProjectRoot,
  expandedProjectRoots,
  onAddProject,
  onPageChange,
  onRemoveProject,
  onSelectProject,
  onToggleProject,
  projects,
}: {
  activePage: AppPage;
  activeProjectRoot: string | null;
  expandedProjectRoots: Set<string>;
  onAddProject: () => void;
  onPageChange: (projectRoot: string, page: AppPage) => void;
  onRemoveProject: (projectRoot: string) => void;
  onSelectProject: (projectRoot: string) => void;
  onToggleProject: (projectRoot: string) => void;
  projects: AgentFlowProjectRef[];
}) {
  const [pendingRemoveProject, setPendingRemoveProject] = useState<AgentFlowProjectRef | null>(null);

  function confirmRemoveProject() {
    if (!pendingRemoveProject) {
      return;
    }
    onRemoveProject(pendingRemoveProject.root);
    setPendingRemoveProject(null);
  }

  return (
    <Sidebar className="v16-project-tree" aria-label="项目导航">
      <button className="v16-add-project-button" onClick={onAddProject} type="button">
        <Plus size={14} />
        <span>添加项目</span>
      </button>
      <div className="v16-project-tree-scroll">
        <p className="v16-project-tree-label">所有项目</p>
        {projects.length ? (
          <nav className="v16-project-tree-list">
            {projects.map((project) => {
              const expanded = expandedProjectRoots.has(project.root);
              const active = project.root === activeProjectRoot;
              return (
                <div className="v16-project-group" key={project.root}>
                  <div className={active ? "v16-project-node active" : "v16-project-node"}>
                    <button
                      aria-expanded={expanded}
                      aria-label={`${expanded ? "收起" : "展开"}${project.name}`}
                      className="v16-project-toggle"
                      data-agentflow-project-root={project.root}
                      data-agentflow-project-toggle
                      onClick={() => onToggleProject(project.root)}
                      type="button"
                    >
                      {expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
                    </button>
                    <button
                      className="v16-project-label"
                      data-agentflow-project-root={project.root}
                      data-agentflow-project-select
                      onClick={() => onSelectProject(project.root)}
                      title={project.root}
                      type="button"
                    >
                      <span
                        aria-label={projectStatusLabel(project.status)}
                        className={`v16-project-status-dot ${project.status}`}
                        role="img"
                        title={projectStatusLabel(project.status)}
                      />
                      <span className="v16-project-name">{project.name}</span>
                    </button>
                    <button
                      aria-label={`从列表移除 ${project.name}`}
                      className="v16-project-remove"
                      data-agentflow-project-remove
                      data-agentflow-project-root={project.root}
                      onClick={() => setPendingRemoveProject(project)}
                      title="从列表移除，不删除本地文件"
                      type="button"
                    >
                      <X size={13} />
                    </button>
                  </div>
                  {expanded ? (
                    <div className="v16-project-page-list">
                      {pages.map((page) => {
                        const Icon = page.icon;
                        return (
                          <button
                            className={active && page.id === activePage ? "active" : ""}
                            data-agentflow-page-id={page.id}
                            data-agentflow-project-root={project.root}
                            key={`${project.root}-${page.id}`}
                            onClick={() => onPageChange(project.root, page.id)}
                            type="button"
                          >
                            <Icon size={14} />
                            <span>{page.label}</span>
                          </button>
                        );
                      })}
                    </div>
                  ) : null}
                </div>
              );
            })}
          </nav>
        ) : null}
      </div>
      {pendingRemoveProject ? (
        <div className="v16-project-remove-dialog" role="dialog" aria-modal="false" aria-label="从列表移除项目">
          <strong>从列表移除 {pendingRemoveProject.name}</strong>
          <p>这只会把项目从 AgentFlow 侧边栏移除，不会删除你的本地文件。</p>
          <div>
            <button onClick={() => setPendingRemoveProject(null)} type="button">
              取消
            </button>
            <button
              className="danger"
              data-agentflow-project-remove-confirm
              data-agentflow-project-root={pendingRemoveProject.root}
              onClick={confirmRemoveProject}
              type="button"
            >
              从列表移除
            </button>
          </div>
        </div>
      ) : null}
    </Sidebar>
  );
}

function Toolbar({
  activePage,
  onRefresh,
  onSearchChange,
  taskSearch,
}: {
  activePage: AppPage;
  onRefresh: () => void;
  onSearchChange: (value: string) => void;
  taskSearch: string;
}) {
  return (
    <TopBar className="v16-toolbar">
      <div>
        <h1>{pageTitle(activePage)}</h1>
      </div>
      <div className="v16-toolbar-actions">
        {activePage === "tasks" ? (
          <label className="v16-inline-search">
            <Search size={14} />
            <input
              aria-label="搜索任务"
              onChange={(event) => onSearchChange(event.target.value)}
              placeholder="搜索任务"
              value={taskSearch}
            />
          </label>
        ) : null}
        <button aria-label={`刷新${pageTitle(activePage)}`} className="v16-icon-button" onClick={onRefresh} type="button">
          <RefreshCw size={16} />
        </button>
      </div>
    </TopBar>
  );
}

function EmptyProjectPage({ onAddProject }: { onAddProject: () => void }) {
  return (
    <section className="v16-page v16-empty-project-page" data-agentflow-page="empty-project">
      <Panel className="v16-empty-project-card">
        <div className="v16-empty-project-mark" aria-hidden="true">
          <FolderOpen size={18} />
        </div>
        <div className="v16-empty-project-copy">
          <p className="v16-empty-project-kicker">项目列表</p>
          <h2>还没有项目</h2>
          <p>添加一个本地项目后，AgentFlow 会准备任务、文件、交付和审计工作区。</p>
          <p className="v16-empty-project-note">移除项目不会删除你的本地文件。</p>
        </div>
        <ActionBar>
          <ActionButton onClick={onAddProject} variant="primary">
            添加本地项目
          </ActionButton>
        </ActionBar>
      </Panel>
    </section>
  );
}

function ProjectAvailabilityPage({
  error,
  onAddProject,
  projectName,
  status,
}: {
  error?: string | null;
  onAddProject: () => void;
  projectName: string;
  status: "loading" | "error" | "missing";
}) {
  const content = {
    error: {
      body: "请检查项目路径是否还存在，或重新添加项目。",
      title: "项目读取失败",
    },
    loading: {
      body: "正在准备 AgentFlow 工作区。",
      title: "正在读取项目",
    },
    missing: {
      body: "这个项目可能被移动或删除了。",
      title: "项目路径不存在",
    },
  }[status];

  return (
    <section className="v16-page v16-empty-project-page" data-agentflow-page="project-unavailable">
      <Panel title={content.title}>
        <p>
          <strong>{projectName}</strong>
        </p>
        <p>{content.body}</p>
        {error ? <p>{error}</p> : null}
        {status !== "loading" ? (
          <ActionBar>
            <ActionButton onClick={onAddProject} variant="primary">
              重新添加项目
            </ActionButton>
          </ActionBar>
        ) : null}
      </Panel>
    </section>
  );
}

function ProjectHomePage({
  connectedProvider,
  nextStep,
  onOpenAudit,
  onOpenDelivery,
  onOpenFiles,
  onOpenTasks,
  outputBundle,
  outputStatusState,
  projectPanelState,
  projectFilesState,
  initializationState,
  projectName,
  projectRoot,
  selectedTask,
  stateStatusState,
  workspaceData,
}: {
  connectedProvider: Provider;
  nextStep: NextStepViewModel;
  onOpenAudit: () => void;
  onOpenDelivery: () => void;
  onOpenFiles: () => void;
  onOpenTasks: () => void;
  outputBundle: OutputBundleState;
  outputStatusState: OutputStatusState;
  projectPanelState: ProjectPanelState;
  projectFilesState: ProjectFilesState;
  initializationState: ProjectInitializationState;
  projectName: string;
  projectRoot: string | null;
  selectedTask: V1Issue | null;
  stateStatusState: StateStatusState;
  workspaceData: WorkspaceDataState;
}) {
  const panelStatus = projectPanelState.status;
  const outputSummary = outputStatusState.status?.summary;
  const filesMode = isBrowserPreviewRuntime() ? "浏览器预览" : "客户端真实读取";
  const recentActivities = buildRecentActivities(workspaceData, outputBundle, initializationState.status, outputSummary);

  return (
    <section className="v16-page v16-home-page" data-agentflow-page="workbench">
      <section className="v16-home-columns" aria-label="工作台总览">
        <Panel className="v16-home-column" title="项目状态">
          <div className="v16-status-stack">
            <HomeStatusItem
              detail={workflowStageText(stateStatusState.status?.currentStage)}
              label="项目"
              status={stateStatusState.status?.currentStage ? "就绪" : "等待"}
              title={projectName}
            />
            <HomeStatusItem
              detail={initializationDetail(initializationState)}
              label="初始化"
              status={initializationState.status?.initialized ? "已就绪" : "等待"}
              title={initializationTitle(initializationState.status)}
            />
            <HomeStatusItem
              detail={`${connectedProvider} · 本地只读客户端`}
              label="工作台 Shell"
              status={projectRoot ? "已就绪" : "未选择项目"}
              title="AgentFlow"
            />
            <HomeStatusItem
              detail={`${filesMode} · 只读`}
              label="项目文件"
              status={projectFilesState.snapshot ? "已读取" : "等待读取"}
              title={projectFilesState.snapshot?.projectRoot ? projectNameFromPath(projectFilesState.snapshot.projectRoot) : "文件阅读器"}
            />
          </div>
          <ActionBar>
            <ActionButton onClick={onOpenFiles} variant="secondary">打开文件页</ActionButton>
          </ActionBar>
        </Panel>

        <Panel className="v16-home-column v16-home-task-column" title="当前任务">
          {selectedTask ? (
            <button className="v16-current-task-card" onClick={onOpenTasks} type="button">
              <span className="v16-current-task-meta">
                <span>{selectedTask.id}</span>
                <RiskBadge risk={selectedTask.riskLevel || "low"} />
              </span>
              <strong>{selectedTask.title}</strong>
              <dl>
                <div>
                  <dt>状态</dt>
                  <dd>{displayStatusLabelZh(selectedTask.displayStatus)}</dd>
                </div>
              </dl>
            </button>
          ) : (
            <div className="v16-home-empty">
              <strong>还没有任务</strong>
              <span>先确认需求，生成任务合约。</span>
            </div>
          )}
          <p className="v16-home-next-step">{nextStep.description}</p>
          <ActionBar>
            <ActionButton disabled={!selectedTask} onClick={onOpenTasks} variant="primary">
              进入任务页
            </ActionButton>
          </ActionBar>
        </Panel>

        <Panel className="v16-home-column" title="最近活动">
          <div className="v16-activity-list">
            {recentActivities.map((activity) => (
              <button
                key={activity.id}
                onClick={activity.target === "delivery" ? onOpenDelivery : activity.target === "audit" ? onOpenAudit : onOpenTasks}
                type="button"
              >
                <strong>{activity.title}</strong>
                <span>{activity.detail}</span>
              </button>
            ))}
          </div>
        </Panel>
      </section>
      <CodexRoleGuideCard defaultOpen={!selectedTask} />
    </section>
  );
}

function CodexRoleGuideCard({ defaultOpen }: { defaultOpen: boolean }) {
  const [copyFeedback, setCopyFeedback] = useState<string | null>(null);

  async function copyStartupInstruction(guide: CodexRoleGuide) {
    try {
      await navigator.clipboard.writeText(guide.startupInstruction);
      setCopyFeedback(`已复制。请粘贴到 ${guide.threadName} 线程。`);
    } catch {
      setCopyFeedback("复制失败。请手动复制启动指令。");
    }
  }

  return (
    <details className="v16-codex-role-guide" open={defaultOpen}>
      <summary>
        <span>
          <strong>Agent 角色使用说明</strong>
          <small>AgentFlow 不直接控制执行过程。你需要按角色开线程，每个线程只做一种工作。</small>
        </span>
        <StatusBadge status="idle">本地说明</StatusBadge>
      </summary>
      <div className="v16-codex-role-guide-body">
        <p className="v16-codex-role-warning">
          不要让同一个执行线程一会儿写代码、一会儿审计。这样容易混淆边界。
        </p>
        <div className="v16-codex-role-grid">
          {codexRoleGuides.map((guide) => (
            <article className="v16-codex-role-card" key={guide.role}>
              <span>{guide.englishName}</span>
              <strong>{guide.title}</strong>
              <p>{guide.summary}</p>
              <small>线程名：{guide.threadName}</small>
              <ActionButton onClick={() => copyStartupInstruction(guide)} variant="secondary">
                复制 {guide.englishName} 启动指令
              </ActionButton>
            </article>
          ))}
        </div>
        {copyFeedback ? <p className="v16-feedback">{copyFeedback}</p> : null}
      </div>
    </details>
  );
}

function HomeStatusItem({
  detail,
  label,
  status,
  title,
}: {
  detail: string;
  label: string;
  status: string;
  title: string;
}) {
  return (
    <article className="v16-home-status-item">
      <p>{label}</p>
      <strong>{title}</strong>
      <span>{detail}</span>
      <StatusBadge status={status.includes("等待") || status.includes("未") ? "idle" : "ready"}>{status}</StatusBadge>
    </article>
  );
}

function TasksPage({
  actionFeedback,
  actions,
  copyState,
  onSelectProjectGroup,
  onTaskAction,
  onSelectTask,
  selectedProjectGroup,
  selectedTask,
  projectRoot,
  suggestions,
  taskTree,
  tasks,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  onSelectProjectGroup: (projectId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  onSelectTask: (taskId: string) => void;
  projectRoot: string | null;
  selectedProjectGroup: TaskProjectGroup | null;
  selectedTask: V1Issue | null;
  suggestions: ProjectInitializationContext[];
  taskTree: TaskProjectTreeViewModel | null;
  tasks: V1Issue[];
}) {
  return (
    <section className="v16-page v16-tasks-page" data-agentflow-page="tasks">
      <TaskList
        actionFeedback={actionFeedback}
        actions={actions}
        copyState={copyState}
        onSelectProjectGroup={onSelectProjectGroup}
        onSelectTask={onSelectTask}
        onTaskAction={onTaskAction}
        projectRoot={projectRoot}
        selectedProjectGroup={selectedProjectGroup}
        selectedTask={selectedTask}
        suggestions={suggestions}
        taskTree={taskTree}
        tasks={tasks}
      />
    </section>
  );
}

function TaskList({
  actionFeedback,
  actions,
  copyState,
  onSelectProjectGroup,
  onSelectTask,
  onTaskAction,
  projectRoot,
  selectedProjectGroup,
  selectedTask,
  suggestions,
  taskTree,
  tasks,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  onSelectProjectGroup: (projectId: string) => void;
  onSelectTask: (taskId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  projectRoot: string | null;
  selectedProjectGroup: TaskProjectGroup | null;
  selectedTask: V1Issue | null;
  suggestions: ProjectInitializationContext[];
  taskTree: TaskProjectTreeViewModel | null;
  tasks: V1Issue[];
}) {
  const showContextSuggestions = !tasks.length && suggestions.length > 0;
  const taskIdSet = useMemo(() => new Set(tasks.map((task) => task.id)), [tasks]);
  const taskTreeProjectIds = taskTree?.groups.map((group) => group.id).join("|") ?? "";
  const taskTreeStorageKey = `agentflow.task-project-tree.expanded.v1:${projectRoot ?? "no-project"}`;
  const [expandedProjectIds, setExpandedProjectIds] = useState<Set<string>>(() => new Set());

  useEffect(() => {
    if (!taskTree) {
      setExpandedProjectIds(new Set());
      return;
    }
    const projectIds = taskTree.groups.map((group) => group.id);
    const saved = window.localStorage.getItem(taskTreeStorageKey);
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed)) {
          setExpandedProjectIds(new Set(parsed.filter((id): id is string => projectIds.includes(id))));
          return;
        }
      } catch {
        // Fall back to the default expanded state.
      }
    }
    setExpandedProjectIds(new Set(projectIds));
  }, [taskTree, taskTreeProjectIds, taskTreeStorageKey]);

  const visibleTaskGroups = useMemo(
    () =>
      taskTree?.groups
        .map((group) => ({
          ...group,
          issues: group.issues.filter((issue) => taskIdSet.has(issue.id)),
        }))
        .filter((group) => group.issues.length || group.missingIssueIds.length || group.counts.issueCount === 0) ?? [],
    [taskIdSet, taskTree],
  );
  const visibleUngroupedIssues = useMemo(
    () => taskTree?.ungroupedIssues.filter((issue) => taskIdSet.has(issue.id)) ?? [],
    [taskIdSet, taskTree],
  );
  const visibleTaskCount =
    visibleTaskGroups.reduce((total, group) => total + group.issues.length, 0) + visibleUngroupedIssues.length;
  const countLabel = showContextSuggestions
    ? `${suggestions.length} 条`
    : taskTree
      ? `${visibleTaskCount} 项`
      : `${tasks.length} 项`;

  const toggleProjectGroup = (projectId: string) => {
    setExpandedProjectIds((current) => {
      const next = new Set(current);
      if (next.has(projectId)) {
        next.delete(projectId);
      } else {
        next.add(projectId);
      }
      window.localStorage.setItem(taskTreeStorageKey, JSON.stringify([...next]));
      return next;
    });
  };

  return (
    <div className="v16-task-list-layout" aria-label="任务流转">
      <aside className="v16-list-pane v16-task-queue-pane" aria-label="任务流转">
        <header>
          <h2>任务流转</h2>
          <span>{countLabel}</span>
        </header>
        <div className="v16-task-queue-items">
          {taskTree && visibleTaskGroups.length
            ? visibleTaskGroups.map((group) => (
                <TaskProjectGroupRow
                  expanded={expandedProjectIds.has(group.id)}
                  group={group}
                  key={group.id}
                  onSelectProjectGroup={onSelectProjectGroup}
                  onSelectTask={onSelectTask}
                  onToggle={toggleProjectGroup}
                  selectedProjectId={selectedProjectGroup?.id ?? null}
                  selectedTaskId={selectedTask?.id ?? null}
                />
              ))
            : null}
          {taskTree && visibleUngroupedIssues.length ? (
            <UngroupedIssueSection
              issues={visibleUngroupedIssues}
              onSelectTask={onSelectTask}
              selectedTaskId={selectedTask?.id ?? null}
            />
          ) : null}
          {taskTree?.warnings.length ? (
            <div className="v16-task-tree-warnings" role="status">
              {taskTree.warnings.slice(0, 3).map((warning) => (
                <p key={`${warning.kind}-${warning.projectId ?? ""}-${warning.issueId ?? ""}-${warning.message}`}>
                  {warning.message}
                </p>
              ))}
            </div>
          ) : null}
          {!taskTree && tasks.length
            ? tasks.map((task) => (
                <button
                  className={task.id === selectedTask?.id ? "v16-task-queue-row active" : "v16-task-queue-row"}
                  key={task.id}
                  onClick={() => onSelectTask(task.id)}
                  title={`${task.id} ${task.title}`}
                  type="button"
                >
                  <span className="v16-task-queue-main">
                    <span className="v16-list-item-id">{task.id}</span>
                    <span className="v16-task-queue-title-line">
                      <span>{task.title}</span>
                    </span>
                  </span>
                  <span className="v16-task-queue-state">
                    <StatusBadge
                      status={statusChipForDisplayStatus(task.displayStatus)}
                    >
                      {displayStatusLabelZh(task.displayStatus)}
                    </StatusBadge>
                  </span>
                </button>
              ))
            : null}
          {showContextSuggestions
            ? suggestions.map((suggestion) => (
                <article className="v16-context-suggestion" key={suggestion.id}>
                  <span className="v16-task-queue-main">
                    <strong className="v16-list-item-id">{suggestion.id}</strong>
                    <span className="v16-task-queue-title-line">
                      <span>{suggestion.title}</span>
                    </span>
                    <small>{suggestion.summary}</small>
                  </span>
                </article>
              ))
            : null}
          {!visibleTaskCount && !tasks.length && !suggestions.length ? (
            <p className="v16-empty-text v16-list-empty-state">还没有任务。</p>
          ) : null}
        </div>
      </aside>
      <TaskDetail
        actionFeedback={actionFeedback}
        actions={actions}
        copyState={copyState}
        onTaskAction={onTaskAction}
        onSelectTask={onSelectTask}
        selectedProjectGroup={selectedProjectGroup}
        suggestions={showContextSuggestions ? suggestions : []}
        task={selectedTask}
        taskTreeSelection={taskTree?.selection ?? null}
      />
    </div>
  );
}

function TaskProjectGroupRow({
  expanded,
  group,
  onSelectProjectGroup,
  onSelectTask,
  onToggle,
  selectedProjectId,
  selectedTaskId,
}: {
  expanded: boolean;
  group: TaskProjectGroup;
  onSelectProjectGroup: (projectId: string) => void;
  onSelectTask: (taskId: string) => void;
  onToggle: (projectId: string) => void;
  selectedProjectId: string | null;
  selectedTaskId: string | null;
}) {
  const progress = group.counts.issueCount
    ? `${group.counts.doneIssueCount}/${group.counts.issueCount}`
    : "0/0";
  const risk = groupRiskLevel(group.issues);
  const selected = group.id === selectedProjectId;

  return (
    <section className="v16-task-project-group" aria-label={group.title}>
      <button
        aria-expanded={expanded}
        className={selected ? "v16-task-project-row active" : "v16-task-project-row"}
        onClick={() => {
          onSelectProjectGroup(group.id);
          onToggle(group.id);
        }}
        title={`${group.title} ${progress}`}
        type="button"
      >
        {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        <span className={`v16-task-project-status ${riskStatusDotClass(risk)}`} aria-hidden="true" />
        <span className="v16-task-project-main">
          <span className="v16-task-project-title">{group.title}</span>
        </span>
        <span className="v16-task-project-meta">
          <span>{progress}</span>
        </span>
      </button>
      {expanded ? (
        <div className="v16-task-project-children">
          {group.issues.map((issue) => (
            <TaskIssueNodeRow
              issue={issue}
              key={issue.id}
              onSelectTask={onSelectTask}
              selected={issue.id === selectedTaskId}
            />
          ))}
          {group.missingIssueIds.map((issueId) => (
            <p className="v16-task-tree-warning" key={issueId}>
              缺失引用：{issueId}
            </p>
          ))}
          {!group.issues.length && !group.missingIssueIds.length ? (
            <p className="v16-empty-text v16-task-project-empty">项目下还没有任务。</p>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}

function UngroupedIssueSection({
  issues,
  onSelectTask,
  selectedTaskId,
}: {
  issues: TaskIssueNode[];
  onSelectTask: (taskId: string) => void;
  selectedTaskId: string | null;
}) {
  return (
    <section className="v16-task-project-group v16-task-ungrouped-section" aria-label="未归属任务">
      <div className="v16-task-project-row v16-task-ungrouped-row">
        <span className="v16-task-project-main">
          <span className="v16-task-project-title">未归属任务</span>
        </span>
        <span className="v16-task-project-meta">{issues.length} 项</span>
      </div>
      <div className="v16-task-project-children">
        {issues.map((issue) => (
          <TaskIssueNodeRow
            issue={issue}
            key={issue.id}
            onSelectTask={onSelectTask}
            selected={issue.id === selectedTaskId}
          />
        ))}
      </div>
    </section>
  );
}

function TaskIssueNodeRow({
  issue,
  onSelectTask,
  selected,
}: {
  issue: TaskIssueNode;
  onSelectTask: (taskId: string) => void;
  selected: boolean;
}) {
  return (
    <button
      className={selected ? "v16-task-queue-row v16-task-node-row active" : "v16-task-queue-row v16-task-node-row"}
      onClick={() => onSelectTask(issue.id)}
      title={`${issue.id} ${issue.title} ${issueCategoryLabelZh(issue.issueCategory)} ${agentRoleLabelZh(issue.requiredAgentRole)}`}
      type="button"
    >
      <span className="v16-task-queue-main">
        <span className="v16-list-item-id">{issue.id}</span>
        <span className="v16-task-queue-title-line">
          <span>{issue.title}</span>
        </span>
      </span>
      <span className="v16-task-queue-state">
        <StatusBadge status={statusChipForDisplayStatus(issue.displayStatus)}>
          {displayStatusLabelZh(issue.displayStatus)}
        </StatusBadge>
      </span>
    </button>
  );
}

function TaskDetail({
  actionFeedback,
  actions,
  copyState,
  onSelectTask,
  onTaskAction,
  selectedProjectGroup,
  suggestions,
  task,
  taskTreeSelection,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  onSelectTask: (taskId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  selectedProjectGroup: TaskProjectGroup | null;
  suggestions: ProjectInitializationContext[];
  task: V1Issue | null;
  taskTreeSelection: TaskProjectTreeViewModel["selection"] | null;
}) {
  if (selectedProjectGroup) {
    return (
      <ProjectSummaryReader
        group={selectedProjectGroup}
        onSelectTask={onSelectTask}
        treeSelection={taskTreeSelection}
      />
    );
  }

  if (!task) {
    if (suggestions.length) {
      return (
        <aside className="v16-detail-pane" aria-label="最近项目记录">
          <header>
            <p className="v16-kicker">上下文建议</p>
            <h2>从最近记录继续</h2>
            <p>这些只是项目上下文，还不是已确认 Issue。</p>
          </header>
          <div className="v16-detail-document">
            <SectionList
              title="可整理的方向"
              items={suggestions.slice(0, 5).map((suggestion) => `${suggestion.title}：${suggestion.summary}`)}
            />
            <SectionList
              title="下一步"
              items={["先把其中一个方向整理成 SPEC，再生成 Issue。确认后才能进入执行。"]}
            />
          </div>
        </aside>
      );
    }
    return (
      <section className="v16-detail-pane v16-empty-detail-pane" aria-label="任务详情空态">
        <header>
          <p className="v16-kicker">任务详情</p>
          <h2>还没有任务</h2>
          <StatusBadge status="idle">等待需求</StatusBadge>
        </header>
        <div className="v16-detail-document">
          <SectionList title="下一步" items={["先确认需求，再生成 Issue。"]} />
          <SectionList title="任务来源" items={["任务只来自 AgentFlow input issues。GitHub 提交记录不会自动变成任务。"]} />
        </div>
      </section>
    );
  }

  return (
    <IssueContractReader
      actionFeedback={actionFeedback}
      actions={actions}
      copyState={copyState}
      onTaskAction={onTaskAction}
      task={task}
    />
  );
}

function ProjectSummaryReader({
  group,
  onSelectTask,
  treeSelection,
}: {
  group: TaskProjectGroup;
  onSelectTask: (taskId: string) => void;
  treeSelection: TaskProjectTreeViewModel["selection"] | null;
}) {
  const risk = groupRiskLevel(group.issues);
  const recommendedIssue = projectRecommendedIssue(group, treeSelection);
  const reviewIssueCount = group.issues.filter((issue) => issue.displayStatus === "review").length;
  const readyIssueCount = group.issues.filter((issue) => issue.displayStatus === "ready").length;
  const highRiskIssueCount = group.issues.filter((issue) => riskToneKey(issue.riskLevel) === "high").length;
  const projectStatus = projectDisplayStatusForGroup(group);

  return (
    <aside className="v16-detail-pane" aria-label="项目摘要">
      <header>
        <h2>{group.title}</h2>
        <p>{group.summary || group.objective || group.id}</p>
        <div className="v16-detail-meta-strip">
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">状态</span>
            <StatusBadge status={statusChipForProjectStatus(projectStatus)}>{projectDisplayStatusLabelZh(projectStatus)}</StatusBadge>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">风险</span>
            <strong className={`v16-risk-text ${riskTextClass(risk)}`}>{displayRiskTextZh(risk)}</strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">来源 SPEC</span>
            <strong className="v16-role-text">{group.sourceSpecId ?? "未记录"}</strong>
          </span>
        </div>
      </header>
      <div className="v16-detail-document">
        <div className="v16-summary-grid">
          <MetricCard detail={`${group.counts.doneIssueCount} 已完成`} label="任务" value={group.counts.issueCount} />
          <MetricCard detail={`${readyIssueCount} 就绪`} label="进行中" value={group.counts.activeIssueCount} />
          <MetricCard detail={`${reviewIssueCount} 待审阅`} label="审计任务" value={group.counts.auditIssueCount} />
          <MetricCard detail={`${highRiskIssueCount} 高风险`} label="风险" value={displayRiskTextZh(risk)} />
        </div>
        <DescriptionList
          items={[
            ["项目 ID", group.id],
            ["来源 SPEC", group.sourceSpecId ?? "未记录"],
            ["当前建议任务", recommendedIssue ? `${recommendedIssue.id} · ${recommendedIssue.title}` : "暂无"],
            ["任务进度", `${group.counts.doneIssueCount}/${group.counts.issueCount}`],
          ]}
        />
        <SectionList title="目标" items={[group.objective || group.summary || group.title]} />
        <SectionList title="范围" items={group.project.scope} />
        <SectionList title="非目标" items={group.project.nonGoals} />
        <SectionList title="任务进度" items={projectProgressItems(group)} />
        <SectionList title="风险摘要" items={projectRiskSummaryItems(group)} />
        <SectionList title="当前建议任务" items={projectRecommendedIssueItems(recommendedIssue)} />
        <SectionList title="依赖摘要" items={projectDependencySummaryItems(group)} />
        {group.warnings.length || group.missingIssueIds.length ? (
          <SectionList title="缺失引用" items={projectWarningItems(group)} />
        ) : null}
      </div>
      <ActionBar sticky>
        <ActionButton
          disabled={!recommendedIssue}
          onClick={() => {
            if (recommendedIssue) {
              onSelectTask(recommendedIssue.id);
            }
          }}
          variant="primary"
        >
          查看建议任务
        </ActionButton>
      </ActionBar>
    </aside>
  );
}

function IssueContractReader({
  actionFeedback,
  actions,
  copyState,
  onTaskAction,
  task,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  task: V1Issue;
}) {
  const handoffError = taskHandoffValidationError(task);
  const detailDescriptionItems = [
    ...taskAuditDescriptionItems(task),
    ...(task.auditTrigger ? [["触发来源", auditTriggerLabel(task.auditTrigger)] as [string, string]] : []),
  ];
  const outputItems = taskOutputItems(task);

  return (
    <aside className="v16-detail-pane" aria-label="Issue 合约">
      <header>
        <h2>{task.id}</h2>
        <p>{task.title}</p>
        <div className="v16-detail-meta-strip">
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">类型：</span>
            <strong className="v16-role-text">{issueCategoryLabelZh(task.issueCategory)}</strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">状态</span>
            <StatusBadge status={statusChipForDisplayStatus(task.displayStatus)}>
              {displayStatusLabelZh(task.displayStatus)}
            </StatusBadge>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">风险</span>
            <strong className={`v16-risk-text ${riskTextClass(task.riskLevel)}`}>
              {displayRiskTextZh(task.riskLevel)}
            </strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">角色</span>
            <strong className="v16-role-text">{agentRoleLabelZh(task.requiredAgentRole)}</strong>
          </span>
        </div>
      </header>
      <div className="v16-detail-document">
        {detailDescriptionItems.length ? <DescriptionList items={detailDescriptionItems} /> : null}
        <SectionList title="目标" items={[task.goal || task.title]} />
        <SectionList title="范围" items={task.scope} />
        <SectionList title="非目标" items={task.nonGoals} />
        <SectionList title="验收标准" items={task.acceptanceCriteria} />
        <SectionList title="证据要求" items={task.evidenceRequired} />
        <SectionList title="验证命令" items={task.validationCommands} />
        {task.issueCategory === "spec" ? <SectionList title="执行流程" items={taskExecutionPipelineItems(task)} /> : null}
        <SectionList title="相关文件" items={task.allowedFiles} />
        <SectionList title="输出目标" items={outputItems} />
        <details className="v16-task-package">
          <summary>Agent 任务包</summary>
          <CopyableCodeBlock
            content={handoffError ?? buildCodexHandoff(task)}
            maxHeight={210}
            title="Agent 任务包"
          />
        </details>
      </div>
      {actionFeedback ? <p className="v16-feedback">{actionFeedback}</p> : null}
      {actions.length ? (
        <ActionBar sticky>
          {actions.map((action, index) => (
            <ActionButton
              disabled={action === "readonly"}
              key={action}
              loading={action === "copy-handoff" && copyState === "loading"}
              onClick={() => onTaskAction(action, task)}
              variant={index === 0 && action !== "readonly" ? "primary" : "secondary"}
            >
              {taskActionDisplayLabel(action, task, copyState)}
            </ActionButton>
          ))}
        </ActionBar>
      ) : null}
    </aside>
  );
}

function FilesPage({
  fileState,
  onChangeViewMode,
  onLoadDirectoryPage,
  onLoadTextRange,
  onSearchFiles,
  onSelectFile,
}: {
  fileState: ProjectFilesState;
  onChangeViewMode: Parameters<typeof ProjectLocalFilesPage>[0]["onChangeViewMode"];
  onLoadDirectoryPage: Parameters<typeof ProjectLocalFilesPage>[0]["onLoadDirectoryPage"];
  onLoadTextRange: Parameters<typeof ProjectLocalFilesPage>[0]["onLoadTextRange"];
  onSearchFiles: Parameters<typeof ProjectLocalFilesPage>[0]["onSearchFiles"];
  onSelectFile: Parameters<typeof ProjectLocalFilesPage>[0]["onSelectFile"];
}) {
  return (
    <section className="v16-page v16-files-page project-local-files-layout" data-agentflow-page="files">
      <ProjectLocalFilesPage
        fileState={fileState}
        onChangeViewMode={onChangeViewMode}
        onLoadDirectoryPage={onLoadDirectoryPage}
        onLoadTextRange={onLoadTextRange}
        onSearchFiles={onSearchFiles}
        onSelectFile={onSelectFile}
      />
    </section>
  );
}

function DeliveryPage({
  onOpenAudit,
  onSelectDelivery,
  outputBundle,
  outputStatusState,
  selectedDeliveryRunId,
  selectedTask,
}: {
  onOpenAudit: () => void;
  onSelectDelivery: (runId: string) => void;
  outputBundle: OutputBundleState;
  outputStatusState: OutputStatusState;
  selectedDeliveryRunId: string | null;
  selectedTask: V1Issue | null;
}) {
  const deliveries = sortOutputEntriesByLatest(outputBundle.outputIndex?.releaseDeliveries ?? []);
  const evidence = sortOutputEntriesByLatest(outputBundle.outputIndex?.evidence ?? []);
  const deliveryInteractionState = buildDeliveryInteractionState(deliveries, selectedDeliveryRunId);
  const selectedDelivery = deliveryInteractionState.selectedDelivery;

  return (
    <section className="v16-page v16-split-page" data-agentflow-page="delivery">
      <DeliveryList
        deliveries={deliveries}
        onSelectDelivery={onSelectDelivery}
        selectedDeliveryRunId={deliveryInteractionState.selectedDeliveryRunId}
      />
      <DeliveryDetail
        audits={outputBundle.auditIndex?.audits ?? []}
        delivery={selectedDelivery}
        evidence={evidence}
        onOpenAudit={onOpenAudit}
        outputStatusState={outputStatusState}
        selectedTask={selectedTask}
      />
    </section>
  );
}

function DeliveryList({
  deliveries,
  onSelectDelivery,
  selectedDeliveryRunId,
}: {
  deliveries: OutputIndexEntry[];
  onSelectDelivery: (runId: string) => void;
  selectedDeliveryRunId: string | null;
}) {
  return (
    <aside className="v16-list-pane" aria-label="交付列表">
      <header>
        <h2>交付列表</h2>
        <span>{deliveries.length} 项</span>
      </header>
	      {deliveries.length ? (
	        <div className="v16-list-items">
	          {deliveries.map((delivery) => (
            <button
              className={delivery.runId === selectedDeliveryRunId ? "v16-list-item active" : "v16-list-item"}
              key={delivery.runId}
              onClick={() => onSelectDelivery(delivery.runId)}
              title={delivery.runId}
              type="button"
            >
              <span className="v16-list-item-main">
                <strong>{deliveryDisplayId(delivery.runId)}</strong>
                <span>{delivery.issueId ? `关联任务：${delivery.issueId}` : "未记录任务"}</span>
              </span>
              <small>{artifactStatusLabel(delivery.status)}</small>
              <time>{formatTimestamp(delivery.updatedAt)}</time>
            </button>
	          ))}
	        </div>
	      ) : (
	        <div className="v16-list-items">
	          <p className="v16-empty-text v16-list-empty-state">还没有交付。</p>
	        </div>
	      )}
    </aside>
  );
}

function DeliveryDetail({
  audits,
  delivery,
  evidence,
  onOpenAudit,
  outputStatusState,
  selectedTask,
}: {
  audits: AuditIndexEntry[];
  delivery: OutputIndexEntry | null;
  evidence: OutputIndexEntry[];
  onOpenAudit: () => void;
  outputStatusState: OutputStatusState;
  selectedTask: V1Issue | null;
}) {
  const deliveryAudit = delivery ? findAuditForDelivery(audits, delivery.runId) : null;
  const auditDisplay = deliveryAuditStatus(delivery, deliveryAudit);
  return (
    <section className={delivery ? "v16-detail-pane" : "v16-detail-pane v16-empty-detail-pane"} aria-label="交付详情">
      <header>
        <p className="v16-kicker">交付包</p>
        <h2>{delivery ? `交付包：${deliveryDisplayId(delivery.runId)}` : "还没有交付材料"}</h2>
        <StatusBadge status={delivery ? "ready" : "idle"}>
          {delivery ? artifactStatusLabel(delivery.status) : "等待写回"}
        </StatusBadge>
      </header>
      <div className="v16-detail-document">
        <div className="v16-summary-grid">
          <MetricCard label="证据" value={outputStatusState.status?.summary.evidence ?? evidence.length} />
          <MetricCard label="验证命令" value={selectedTask?.validationCommands.length ?? 0} />
          <MetricCard label="变更文件" value={selectedTask?.allowedFiles.length ?? 0} />
          <MetricCard label="缺失证据" value={outputStatusState.status?.summary.incompleteEvidence ?? 0} />
        </div>
        <SectionList
          title="交付摘要"
          items={[
            delivery ? "任务合约页面交付记录已生成。" : "等待写回交付记录。",
            "模式：只读",
          ]}
        />
        <SectionList
          title="关联记录"
          items={[
            delivery?.issueId ? `关联任务：${delivery.issueId}` : "关联任务：未记录",
            delivery?.sourceSpecId ? "关联规格：已确认规格" : "关联规格：未记录",
          ]}
        />
        <SectionList title="变更文件" items={selectedTask?.allowedFiles ?? ["等待写回变更文件。"]} />
        <SectionList title="验证命令" items={selectedTask?.validationCommands ?? ["等待验证命令。"]} />
        <SectionList title="验证结果" items={[delivery ? `状态：${artifactStatusLabel(delivery.status)}` : "等待写回。"]} />
        <SectionList title="审计状态" items={[auditDisplay.detail]} />
        <SectionList
          id="v16-delivery-evidence"
          title="证据"
          items={evidence.length ? evidence.map((item) => `${deliveryDisplayId(item.runId)} · ${artifactStatusLabel(item.status)}`) : ["暂无证据。"]}
        />
        <SectionList title="越界检查" items={["普通页面只展示摘要；原始路径和 JSON 在高级页查看。"]} />
      </div>
      <ActionBar sticky>
        <ActionButton disabled={!auditDisplay.canOpenReport} onClick={onOpenAudit} variant="primary">
          {auditDisplay.actionLabel}
        </ActionButton>
        <ActionButton disabled={!evidence.length} onClick={() => document.getElementById("v16-delivery-evidence")?.scrollIntoView({ block: "nearest" })} variant="secondary">
          查看证据
        </ActionButton>
      </ActionBar>
    </section>
  );
}

function AuditPage({
  onSelectAudit,
  outputBundle,
  selectedAuditId,
}: {
  onSelectAudit: (auditId: string) => void;
  outputBundle: OutputBundleState;
  selectedAuditId: string | null;
}) {
  const audits = sortAuditsByLatest(outputBundle.auditIndex?.audits ?? []);
  const auditInteractionState = buildAuditInteractionState(audits, selectedAuditId);
  const selectedReport =
    outputBundle.auditReport?.audit.auditId === auditInteractionState.selectedAuditId ? outputBundle.auditReport : null;
  return (
    <section className="v16-page v16-audit-page" data-agentflow-page="audit">
      <AuditList
        audits={audits}
        onSelectAudit={onSelectAudit}
        selectedAuditId={auditInteractionState.selectedAuditId}
      />
      <AuditReport report={selectedReport} selectedAudit={auditInteractionState.selectedAudit} />
    </section>
  );
}

function AuditList({
  audits,
  onSelectAudit,
  selectedAuditId,
}: {
  audits: AuditIndexEntry[];
  onSelectAudit: (auditId: string) => void;
  selectedAuditId: string | null;
}) {
  return (
    <aside className="v16-list-pane" aria-label="审计列表">
      <header>
        <h2>审计列表</h2>
        <span>{audits.length} 项</span>
      </header>
	      {audits.length ? (
	        <div className="v16-list-items">
	          {audits.map((audit) => (
            <button
              className={audit.auditId === selectedAuditId ? "v16-list-item active" : "v16-list-item"}
              key={audit.auditId}
              onClick={() => onSelectAudit(audit.auditId)}
              title={audit.auditId}
              type="button"
            >
              <span className="v16-list-item-main">
                <strong>{audit.auditId}</strong>
                <span>{auditTriggerLabel(audit.trigger)}</span>
              </span>
              <small>{artifactStatusLabel(audit.status)}</small>
              <time>{formatTimestamp(audit.requestedAt)}</time>
            </button>
	          ))}
	        </div>
	      ) : (
	        <div className="v16-list-items">
	          <p className="v16-empty-text v16-list-empty-state">还没有审计。</p>
	        </div>
	      )}
    </aside>
  );
}

function AuditReport({
  report,
  selectedAudit,
}: {
  report: HumanAuditReport | null;
  selectedAudit: AuditIndexEntry | null;
}) {
  type AuditFindingSummary = {
    findingId?: string;
    id?: string;
    severity?: string;
    summary?: string;
    title?: string;
  };
  const findings = Array.isArray(report?.findings)
    ? (report.findings as AuditFindingSummary[])
    : Array.isArray((report?.findings as { findings?: unknown[] } | undefined)?.findings)
      ? (report?.findings as { findings: AuditFindingSummary[] }).findings
    : [];
  const trigger = report?.request.trigger ?? report?.audit.trigger ?? selectedAudit?.trigger;
  const sourceRunId =
    report?.request.source?.runId ?? report?.audit.sourceRunId ?? selectedAudit?.sourceRunId ?? selectedAudit?.sourceDeliveryId;
  const sourceIssueId =
    report?.request.source?.issueId ?? report?.audit.sourceIssueId ?? selectedAudit?.sourceIssueId;

  return (
    <section className={selectedAudit || report ? "v16-detail-pane" : "v16-detail-pane v16-empty-detail-pane"} aria-label="审计报告详情">
      <header>
        <p className="v16-kicker">审计报告</p>
        <h2>{selectedAudit?.auditId ?? report?.audit.auditId ?? "未登记审计"}</h2>
        <StatusBadge status={selectedAudit || report ? "warning" : "idle"}>
          {artifactStatusLabel(selectedAudit?.status ?? report?.audit.status ?? "未登记")}
        </StatusBadge>
      </header>
      <div className="v16-detail-document">
        <SectionList
          title="触发来源"
          items={[
            auditTriggerLabel(trigger),
            sourceRunId ? `关联交付：${sourceRunId}` : "关联交付：等待 Agent 写入",
            sourceIssueId ? `关联任务：${sourceIssueId}` : "关联任务：等待 Agent 写入",
          ]}
        />
        <SectionList
          title="审计结论"
          items={[
            report?.reportMarkdown.split("\n").slice(0, 3).join(" ") ||
              "暂无审计报告。审计需要通过独立审计任务或人工请求触发。",
          ]}
        />
        <SectionList
          title="发现项"
          items={
            findings.length
              ? findings.map((finding) => `${finding.severity ?? "info"}：${finding.summary ?? finding.title ?? finding.id ?? finding.findingId ?? "发现项"}`)
              : ["暂无发现项。"]
          }
        />
        <HumanSummaryTable title="证据链" rows={summaryRowsFromValue(report?.evidenceMap, "等待交付证据。")} />
        <HumanSummaryTable title="追溯关系" rows={summaryRowsFromValue(report?.traceability, "等待审计追溯关系。")} />
        <SectionList title="范围检查" items={["对照规格、任务、交付和证据。"]} />
        <SectionList title="验证检查" items={["检查验证命令是否记录并通过。"]} />
        <SectionList title="处理建议" items={["等待审计报告写入后展示。"]} />
        <SectionList title="当前版本限制" items={["这里只读展示审计状态和报告，不写处理结果。"]} />
      </div>
    </section>
  );
}

function AdvancedPage({
  agentManualState,
  executeStatusState,
  inputStatusState,
  outputBundle,
  outputStatusState,
  projectFilesState,
  projectPanelState,
  initializationState,
  stateStatusState,
  workspaceData,
}: {
  agentManualState: unknown;
  executeStatusState: unknown;
  inputStatusState: unknown;
  outputBundle: OutputBundleState;
  outputStatusState: OutputStatusState;
  projectFilesState: ProjectFilesState;
  projectPanelState: ProjectPanelState;
  initializationState: ProjectInitializationState;
  stateStatusState: StateStatusState;
  workspaceData: WorkspaceDataState;
}) {
  const categories = [
    { id: "state", label: "状态", value: stateStatusState, files: advancedFilesForCategory("state") },
    { id: "agentRoles", label: "Agent 角色", value: agentRoleRulesDocument(), files: advancedFilesForCategory("agentRoles") },
    { id: "initialization", label: "初始化", value: initializationState, files: advancedFilesForCategory("initialization") },
    { id: "panel", label: "Panel", value: projectPanelState, files: advancedFilesForCategory("panel") },
    { id: "input", label: "Input", value: inputStatusState, files: advancedFilesForCategory("input") },
    { id: "execute", label: "Execute", value: executeStatusState, files: advancedFilesForCategory("execute") },
    { id: "output", label: "Output", value: { outputBundle, outputStatusState }, files: advancedFilesForCategory("output") },
    { id: "audit", label: "Audit", value: outputBundle.auditReport, files: advancedFilesForCategory("audit") },
    { id: "settings", label: "设置", value: { agentManualState, projectFilesState, workspaceData }, files: advancedFilesForCategory("settings") },
  ];
  const [activeCategory, setActiveCategory] = useState(categories[0].id);
  const selectedCategory = categories.find((category) => category.id === activeCategory) ?? categories[0];

  return (
    <section className="v16-page v16-advanced-page" data-agentflow-page="advanced">
      <AdvancedStateViewer
        categories={categories}
        onSelectCategory={setActiveCategory}
        selectedCategory={selectedCategory}
      />
    </section>
  );
}

function AdvancedStateViewer({
  categories,
  onSelectCategory,
  selectedCategory,
}: {
  categories: Array<{ files: Array<{ description: string; name: string }>; id: string; label: string; value: unknown }>;
  onSelectCategory: (categoryId: string) => void;
  selectedCategory: { files: Array<{ description: string; name: string }>; id: string; label: string; value: unknown };
}) {
  return (
    <div className="v16-advanced-layout" aria-label="高级详情">
      <aside className="v16-advanced-nav">
        {categories.map((category) => (
          <button
            className={category.id === selectedCategory.id ? "active" : ""}
            key={category.id}
            onClick={() => onSelectCategory(category.id)}
            type="button"
          >
            {category.label}
          </button>
        ))}
      </aside>
      <section className="v16-advanced-list">
        <h2>{selectedCategory.label}</h2>
        <p>{advancedCategorySummary(selectedCategory.id)}</p>
        <div className="v16-advanced-file-list">
          {selectedCategory.files.map((file) => (
            <article key={file.name}>
              <strong>{file.name}</strong>
              <span>{file.description}</span>
            </article>
          ))}
        </div>
      </section>
      <section className="v16-advanced-reader">
        <header>
          <h2>JSON Reader</h2>
          <p>只读展示。这里不编辑 JSON，不修复状态，不清理锁，不触发审计。</p>
        </header>
        <JsonReader value={selectedCategory.value} />
      </section>
    </div>
  );
}

function JsonReader({ value }: { value: unknown }) {
  return (
    <pre className="v16-json-reader" aria-label="JSON Reader">
      <code>{JSON.stringify(value, null, 2)}</code>
    </pre>
  );
}

function StatusBar({
  appInteractionState,
  projectName,
  projectRoot,
  projectStatus,
  stateStatus,
}: {
  appInteractionState: AppInteractionState;
  projectName: string;
  projectRoot: string | null;
  projectStatus: AgentFlowProjectStatus | null;
  stateStatus: StateStatusState["status"];
}) {
  const projectStatusSummary = statusBarProjectSummary(projectRoot, projectStatus);
  if (!projectRoot) {
    return (
      <FoundationStatusBar className="v16-status-bar" aria-label="底部状态摘要">
        <section>
          <strong>未选择项目</strong>
          <span>本地模式</span>
        </section>
        <section>
          <span>⌘K</span>
        </section>
      </FoundationStatusBar>
    );
  }

  return (
    <FoundationStatusBar className="v16-status-bar" aria-label="底部状态摘要">
      <section>
        <StatusDot status={projectStatusSummary.dot} />
        <span>{projectStatusSummary.label}</span>
        <strong>{projectName}</strong>
        <span>
          <GitBranch size={13} /> local-only
        </span>
      </section>
      <section>
        <span>{workflowStageText(stateStatus?.currentStage)}</span>
        <span>{lifecycleLabel(appInteractionState.lifecycle)}</span>
        <span>⌘K</span>
      </section>
    </FoundationStatusBar>
  );
}

function statusBarProjectSummary(
  projectRoot: string | null,
  projectStatus: AgentFlowProjectStatus | null,
): { dot: StatusChipStatus; label: string } {
  if (!projectRoot) {
    return { dot: "idle", label: "waiting" };
  }
  if (projectStatus === "loading") {
    return { dot: "working", label: "loading" };
  }
  if (projectStatus === "blocked") {
    return { dot: "blocked", label: "blocked" };
  }
  if (projectStatus === "error") {
    return { dot: "failed", label: "error" };
  }
  if (projectStatus === "missing") {
    return { dot: "idle", label: "missing" };
  }
  return { dot: "ready", label: "ready" };
}

function titlebarStatusDotStatus(statusText: string): StatusChipStatus {
  if (statusText === "blocked") {
    return "blocked";
  }
  if (statusText === "error") {
    return "failed";
  }
  if (statusText === "loading" || statusText === "agent-running") {
    return "working";
  }
  if (statusText === "delivered" || statusText === "audit-completed") {
    return "done";
  }
  if (statusText === "workspace-ready" || statusText === "未选择项目 · 本地模式") {
    return "ready";
  }
  if (statusText === "not-authenticated" || statusText === "first-run") {
    return "idle";
  }
  return "warning";
}

function CompanionShell({
  onCheckWriteback,
  onOpenFiles,
  onOpenTasks,
  projectName,
  selectedTask,
}: {
  onCheckWriteback: () => void;
  onOpenFiles: () => void;
  onOpenTasks: () => void;
  projectName: string;
  selectedTask: V1Issue | null;
}) {
  const queueItems = [
    {
      id: "writeback",
      label: "写回",
      title: selectedTask?.title ?? "等待任务写回",
      active: true,
    },
    {
      id: "ready",
      label: "就绪",
      title: selectedTask ? "任务包已准备" : "等待任务包",
      active: false,
    },
    {
      id: "audit",
      label: "审计",
      title: selectedTask ? "等待证据核对" : "等待交付",
      active: false,
    },
  ];

  return (
    <section className="v16-companion-shell" aria-label="协作模式">
      <header>
        <h2>{projectName}</h2>
      </header>
      <div className="v16-companion-queue" aria-label="今日队列">
        <p>今日队列</p>
        {queueItems.map((item) => (
          <button className={item.active ? "active" : ""} key={item.id} type="button">
            <strong>{item.label}</strong>
            <span>{item.title}</span>
          </button>
        ))}
      </div>
      <article className="v16-companion-selected">
        <p>当前任务</p>
        <h3>{selectedTask?.title ?? "还没有选中任务"}</h3>
        <strong>执行助手</strong>
        <span>
          {selectedTask
            ? "等待写回。请确认任务包已经粘贴，然后扫描 .agentflow/output。"
            : "当前没有可交付给执行助手的任务。"}
        </span>
      </article>
      <ActionBar>
        <ActionButton disabled={!selectedTask} onClick={onCheckWriteback} variant="secondary">
          检查写回
        </ActionButton>
        <ActionButton disabled={!selectedTask} onClick={onOpenTasks} variant="primary">
          复制任务
        </ActionButton>
        <ActionButton onClick={onOpenFiles} variant="secondary">
          打开文件
        </ActionButton>
      </ActionBar>
    </section>
  );
}

function ActionBar({ children, sticky = false }: { children: ReactNode; sticky?: boolean }) {
  return <footer className={sticky ? "v16-action-bar sticky" : "v16-action-bar"}>{children}</footer>;
}

function StatusDot({ status }: { status: StatusChipStatus }) {
  return <span className={`v16-status-dot ${status}`} aria-hidden="true" />;
}

function DescriptionList({ items }: { items: Array<[string, string]> }) {
  return (
    <dl className="v16-description-list">
      {items.map(([label, value]) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function SectionList({ id, items, title }: { id?: string; items: string[]; title: string }) {
  return (
    <Section className="v16-section-list" id={id} title={title}>
      <ul>
        {(items.length ? items : ["暂无记录。"]).map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </Section>
  );
}

function HumanSummaryTable({ rows, title }: { rows: Array<[string, string]>; title: string }) {
  return (
    <section className="v16-human-summary">
      <h3>{title}</h3>
      <dl>
        {rows.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function summaryRowsFromValue(value: unknown, emptyText: string): Array<[string, string]> {
  if (!value || typeof value !== "object") {
    return [["状态", emptyText]];
  }
  return Object.entries(value as Record<string, unknown>)
    .slice(0, 6)
    .map(([key, item]) => [humanizeKey(key), summarizeValue(item)]);
}

function initializationTitle(status: ProjectInitializationStatus | null) {
  if (!status) {
    return "正在读取初始化状态";
  }
  if (status.projectKind === "existing") {
    return status.gitContextLoaded ? "已读取最近项目记录" : "已接入现有项目";
  }
  if (status.demoDataCreated) {
    return "新项目示例已准备";
  }
  return "新项目已准备";
}

function initializationDetail(state: ProjectInitializationState) {
  if (state.source === "loading") {
    return "正在读取初始化状态。";
  }
  if (state.error) {
    return state.error;
  }
  if (!state.status) {
    return "等待项目初始化。";
  }
  const warning = state.status.warnings.at(0);
  return warning ? `${state.status.message} ${warning}` : state.status.message;
}

const displayStatusColumns: Array<{ id: IssueDisplayStatus; label: string }> = [
  { id: "backlog", label: "待办" },
  { id: "ready", label: "就绪" },
  { id: "in-progress", label: "进行中" },
  { id: "review", label: "待审阅" },
  { id: "done", label: "已完成" },
  { id: "cancel", label: "已取消" },
];

const displayStatusOrder = new Map(displayStatusColumns.map((column, index) => [column.id, index]));

const buildAgentPipelineStageIds = [
  "github-preflight",
  "implement",
  "sandbox-verify",
  "create-pr",
  "merge-pr",
  "writeback-done",
] as const;

function defaultBuildAgentExecutionPipeline(): ExecutionPipeline {
  return {
    agentRole: "build-agent",
    mergeModes: ["manual-merge", "auto-merge-if-eligible"],
    stages: [
      {
        evidence: [
          "AgentFlow issueId and executionPipeline are the only active task source",
          "no external issue/task/plan/queue/thread/tool state is used as task authority",
          "gh --version",
          "gh auth status",
          "git status --short",
          "git remote -v",
          "gh repo view --json nameWithOwner,defaultBranchRef",
          "cargo build --release --bin agentflow or target/debug/agentflow fallback",
          "target/release/agentflow build-agent complete --help or target/debug/agentflow build-agent complete --help",
        ],
        goal: "确认 Build Agent 只基于 AgentFlow input issue 和 executionPipeline 执行；确认没有把外部 issue、任务、计划、队列、线程或工具状态当作任务源；确认 GitHub 工具、认证、仓库同步、PR 创建和合并能力可用；同时确认当前 AgentFlow CLI 支持 build-agent complete，不能直接复用过期 target/release/agentflow。",
        label: "GitHub 自动化预检",
        required: true,
        stageId: "github-preflight",
      },
      {
        evidence: ["git diff --stat", "changed-files summary"],
        goal: "按 issue 合同在 allowedPaths 内完成代码、配置或测试改动。",
        label: "Agent 执行 issue",
        required: true,
        stageId: "implement",
      },
      {
        evidence: ["validation command records", "browser smoke evidence when applicable", "git diff --check"],
        goal: "在受控本地沙箱中运行验证命令并收集 stdout、stderr、exit code、浏览器或截图证据。",
        label: "沙箱测试与结果验证",
        required: true,
        stageId: "sandbox-verify",
      },
      {
        evidence: ["PR URL", "PR body validation summary", "draft or ready state"],
        goal: "推送任务分支，创建 PR，并把验证结果写入 PR 描述；如果 mergeMode 是 auto-merge-if-eligible，不能停在 Draft PR。",
        label: "创建 PR",
        required: true,
        stageId: "create-pr",
      },
      {
        evidence: ["merge mode", "gh pr ready result", "gh pr merge --auto result", "merge commit or merged PR state"],
        goal: "manual-merge 模式下 PR ready 后等待人合并；auto-merge-if-eligible 模式下执行 gh pr ready、gh pr merge --auto，并轮询到 merged。",
        label: "合并 PR",
        required: true,
        stageId: "merge-pr",
      },
      {
        evidence: [
          "target/release/agentflow build-agent complete --request <completion-request.json> after cargo build --release --bin agentflow",
          "or target/debug/agentflow build-agent complete --request <completion-request.json>",
          "issue status done",
        ],
        goal: "PR 合并后使用预检确认过的新 AgentFlow CLI 调用 build-agent complete，写回 run、evidence、delivery 和任务 Done 状态。",
        label: "写回 Done",
        required: true,
        stageId: "writeback-done",
      },
    ],
    version: "build-agent-execution-pipeline.v1",
  };
}

function buildTaskItems(
  inputIssues: InputIssue[] | null,
  issueStatusIndex: IssueStatusIndex | null,
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null,
  workbench: WorkbenchSnapshot | null,
): V1Issue[] {
  if (inputIssues) {
    return sortTasksByDisplayStatus(inputIssues.map((issue) => inputIssueToV1Issue(issue, issueStatusIndex)));
  }
  if (projectViewModel?.issues.length) {
    return sortTasksByDisplayStatus(
      projectViewModel.issues.map((issue) => withDisplayStatus(issue, issueStatusIndex)),
    );
  }
  return sortTasksByDisplayStatus((workbench?.issues ?? []).map(issueContractToV1Issue));
}

function inputIssueToV1Issue(issue: InputIssue, issueStatusIndex: IssueStatusIndex | null): V1Issue {
  const indexed = issueStatusIndex?.issues.find((item) => item.issueId === issue.issueId);
  const displayStatus = indexed?.displayStatus ?? issue.displayStatus;
  const issueCategory = issue.issueCategory ?? "spec";
  const requiredAgentRole = issue.requiredAgentRole ?? (issueCategory === "audit" ? "audit-agent" : "build-agent");
  const auditId = issue.audit?.auditId ?? (issueCategory === "audit" ? issue.issueId : null);
  const auditOutputDir = issue.audit?.auditOutputDir ?? (auditId ? `.agentflow/output/audit/${auditId}` : null);
  const expectedOutputs = normalizeExpectedOutputs(
    issue.expectedOutputs,
    issueCategory,
    issue.issueId,
    auditId,
    auditOutputDir,
    issue.audit?.expectedOutputs,
    false,
  );
  return {
    acceptanceCriteria: issue.acceptanceCriteria,
    allowedFiles: issue.allowedPaths?.length ? issue.allowedPaths : issue.scope,
    boundary: issue.nonGoals,
    codexInstructions: issue.validationHints,
    dependencies: issue.relations?.blockedBy ?? [],
    displayStatus,
    evidenceRequired: issue.acceptanceCriteria,
    executionPipeline: issueCategory === "spec" ? (issue.executionPipeline ?? defaultBuildAgentExecutionPipeline()) : null,
    expectedOutputs,
    forbiddenActions: issue.forbiddenActions?.length ? issue.forbiddenActions : defaultForbiddenActions(issueCategory),
    forbiddenFiles: issue.forbiddenPaths?.length ? issue.forbiddenPaths : defaultForbiddenPaths(issueCategory),
    goal: issue.summary || issue.title,
    id: issue.issueId,
    auditTrigger: issue.audit?.trigger ?? null,
    auditId,
    auditOutputDir,
    contextPackPath: issue.contextPackPath ?? null,
    createdAt: issue.system?.createdAt ?? null,
    handoffId: issue.handoffId ?? `handoff-${issue.issueId}`,
    issueCategory,
    issuePath: issue.issuePath ?? issue.system?.path ?? `.agentflow/input/issues/${issue.issueId}.json`,
    milestoneId: null,
    nonGoals: issue.nonGoals,
    projectId: issue.projectId ?? null,
    rawStatus: issue.status,
    requiredAgentRole,
    riskLevel: issue.riskLevel || indexed?.riskLevel || "low",
    sourceDeliveryPath: issue.audit?.sourceDeliveryPath ?? null,
    sourceReleaseId: issue.audit?.sourceReleaseId ?? null,
    sourceSpecId: issue.sourceSpecId ?? null,
    sourceSpecPath:
      issue.sourceSpecPath ??
      (issue.sourceSpecId ? `.agentflow/input/specs/approved/${issue.sourceSpecId}/spec.json` : null),
    scope: issue.scope,
    status: issue.status,
    title: issue.title,
    updatedAt: issue.system?.updatedAt ?? issue.system?.createdAt ?? null,
    validationCommands: issue.validationCommands?.length ? issue.validationCommands : issue.validationHints,
  };
}

function issueContractToV1Issue(issue: IssueContract): V1Issue {
  const expectedOutputs = normalizeExpectedOutputs(undefined, "spec", issue.id, null, null);
  return {
    acceptanceCriteria: issue.evidenceRequirements,
    allowedFiles: issue.context.files,
    boundary: issue.nonGoals,
    codexInstructions: issue.executionPlan,
    dependencies: [],
    displayStatus: displayStatusFromLegacyStatus(issue.status),
    evidenceRequired: issue.evidenceRequirements,
    executionPipeline: defaultBuildAgentExecutionPipeline(),
    expectedOutputs,
    forbiddenActions: defaultForbiddenActions("spec"),
    forbiddenFiles: [".agentflow/*", ".codex/*", "agent-artifacts/*"],
    goal: issue.intent,
    id: issue.id,
    auditTrigger: null,
    auditId: null,
    auditOutputDir: null,
    contextPackPath: null,
    createdAt: null,
    handoffId: `handoff-${issue.id}`,
    issueCategory: "spec",
    issuePath: `.agentflow/input/issues/${issue.id}.json`,
    milestoneId: null,
    nonGoals: issue.nonGoals,
    projectId: null,
    rawStatus: issue.status,
    requiredAgentRole: "build-agent",
    riskLevel: "low",
    sourceDeliveryPath: null,
    sourceReleaseId: null,
    sourceSpecId: "legacy-workbench",
    sourceSpecPath: ".agentflow/input/specs/approved/legacy-workbench/spec.json",
    scope: issue.scope,
    status: issue.status,
    title: issue.title,
    updatedAt: null,
    validationCommands: issue.validation.commands,
  };
}

function withDisplayStatus(issue: V1Issue, issueStatusIndex: IssueStatusIndex | null): V1Issue {
  const indexed = issueStatusIndex?.issues.find((item) => item.issueId === issue.id);
  return {
    ...issue,
    displayStatus: indexed?.displayStatus ?? issue.displayStatus ?? displayStatusFromLegacyStatus(issue.status),
  };
}

function normalizeExpectedOutputs(
  outputs: ExpectedOutputs | undefined,
  issueCategory: string,
  issueId: string,
  auditId?: string | null,
  auditOutputDir?: string | null,
  auditOutputs?: ExpectedOutputs | string[] | null,
  allowDefaultOutputs = true,
): ExpectedOutputs {
  if (issueCategory === "audit") {
    const normalizedAuditOutputs = normalizeOutputValue(auditOutputs);
    if (Object.keys(normalizedAuditOutputs).length) {
      return normalizedAuditOutputs;
    }
    const normalizedDirectOutputs = normalizeOutputValue(outputs);
    if (Object.keys(normalizedDirectOutputs).length) {
      return normalizedDirectOutputs;
    }
    const outputDir = auditOutputDir || (auditId ? `.agentflow/output/audit/${auditId}` : "");
    return allowDefaultOutputs && outputDir ? auditExpectedOutputs(outputDir) : {};
  }

  const normalized = normalizeOutputValue(outputs);
  if (Object.keys(normalized).length) {
    return normalized;
  }
  if (!allowDefaultOutputs) {
    return {};
  }
  return {
    evidencePath: `.agentflow/output/evidence/${issueId}.json`,
    executeRunDir: `.agentflow/execute/runs/${issueId}`,
    releaseDeliveryDir: `.agentflow/output/release/${issueId}`,
  };
}

function normalizeOutputValue(outputs?: ExpectedOutputs | string[] | null): ExpectedOutputs {
  if (!outputs) {
    return {};
  }
  if (Array.isArray(outputs)) {
    return Object.fromEntries(
      outputs.map((output) => {
        const key = output.split("/").pop() || output;
        return [key, output];
      }),
    );
  }
  return outputs;
}

function auditExpectedOutputs(auditOutputDir: string): ExpectedOutputs {
  return {
    "audit-report.md": `${auditOutputDir}/audit-report.md`,
    "audit.json": `${auditOutputDir}/audit.json`,
    "evidence-map.json": `${auditOutputDir}/evidence-map.json`,
    "findings.json": `${auditOutputDir}/findings.json`,
    "traceability.json": `${auditOutputDir}/traceability.json`,
  };
}

function defaultForbiddenPaths(issueCategory?: string | null) {
  if (issueCategory === "audit") {
    return [".agentflow/execute/**", ".agentflow/output/release/**", ".agentflow/output/evidence/**"];
  }
  return [".agentflow/output/audit/**", ".agentflow/spec/**", ".agentflow/goal-tree/**"];
}

function defaultForbiddenActions(issueCategory?: string | null) {
  if (issueCategory === "audit") {
    return ["process-spec-issue", "write-source-code", "execute-project-commands", "generate-release-delivery"];
  }
  return ["process-audit-issue", "write-audit-report", "write-audit-findings"];
}

function sortTasksByDisplayStatus(tasks: V1Issue[]) {
  return [...tasks].sort((left, right) => {
    const timeDiff = issueSortTime(right) - issueSortTime(left);
    if (timeDiff) {
      return timeDiff;
    }
    const leftOrder = displayStatusOrder.get(left.displayStatus ?? "backlog") ?? 0;
    const rightOrder = displayStatusOrder.get(right.displayStatus ?? "backlog") ?? 0;
    return leftOrder - rightOrder || left.id.localeCompare(right.id);
  });
}

function issueSortTime(issue: V1Issue) {
  return issue.updatedAt ?? issue.createdAt ?? 0;
}

function sortOutputEntriesByLatest(entries: OutputIndexEntry[]) {
  return [...entries].sort(
    (left, right) => right.updatedAt - left.updatedAt || right.runId.localeCompare(left.runId),
  );
}

function sortAuditsByLatest(audits: AuditIndexEntry[]) {
  return [...audits].sort(
    (left, right) => right.requestedAt - left.requestedAt || right.auditId.localeCompare(left.auditId),
  );
}

function displayStatusFromLegacyStatus(status: string): IssueDisplayStatus {
  const normalized = status.toLowerCase();
  if (normalized.includes("cancel")) {
    return "cancel";
  }
  if (normalized.includes("done") || normalized.includes("complete")) {
    return "done";
  }
  if (normalized.includes("review") || normalized.includes("delivered")) {
    return "review";
  }
  if (normalized.includes("running") || normalized.includes("progress")) {
    return "in-progress";
  }
  if (normalized === "ready" || normalized === "todo" || normalized === "ready-for-execute") {
    return "ready";
  }
  return "backlog";
}

function statusChipForDisplayStatus(status: IssueDisplayStatus = "backlog"): StatusChipStatus {
  const chips: Record<IssueDisplayStatus, StatusChipStatus> = {
    backlog: "idle",
    cancel: "blocked",
    done: "done",
    "in-progress": "working",
    ready: "ready",
    review: "warning",
  };
  return chips[status];
}

type ProjectDisplayStatus = "planned" | "active" | "blocked" | "done" | "canceled";

function projectDisplayStatusForGroup(group: TaskProjectGroup): ProjectDisplayStatus {
  if (!group.issues.length) {
    return normalizeProjectDisplayStatus(group.status);
  }

  const finishedIssues = group.issues.filter((issue) => issue.displayStatus === "done" || issue.displayStatus === "cancel");
  if (finishedIssues.length === group.issues.length) {
    return group.issues.every((issue) => issue.displayStatus === "cancel") ? "canceled" : "done";
  }

  const unfinishedIssues = group.issues.filter((issue) => issue.displayStatus !== "done" && issue.displayStatus !== "cancel");
  if (unfinishedIssues.length && unfinishedIssues.every((issue) => issue.status === "blocked")) {
    return "blocked";
  }

  return "active";
}

function normalizeProjectDisplayStatus(status?: string | null): ProjectDisplayStatus {
  const normalized = (status ?? "planned").toLowerCase();
  if (normalized.includes("cancel")) {
    return "canceled";
  }
  if (normalized.includes("done") || normalized.includes("complete")) {
    return "done";
  }
  if (normalized.includes("blocked")) {
    return "blocked";
  }
  if (normalized.includes("active") || normalized.includes("running") || normalized.includes("progress")) {
    return "active";
  }
  return "planned";
}

function statusChipForProjectStatus(status: ProjectDisplayStatus): StatusChipStatus {
  const chips: Record<ProjectDisplayStatus, StatusChipStatus> = {
    active: "working",
    blocked: "blocked",
    canceled: "blocked",
    done: "done",
    planned: "idle",
  };
  return chips[status];
}

function projectDisplayStatusLabelZh(status: ProjectDisplayStatus) {
  const labels: Record<ProjectDisplayStatus, string> = {
    active: "进行中",
    blocked: "有阻断",
    canceled: "已取消",
    done: "已完成",
    planned: "已计划",
  };
  return labels[status];
}

function projectStatusLabel(status: AgentFlowProjectStatus) {
  const labels: Record<AgentFlowProjectStatus, string> = {
    blocked: "有阻断",
    error: "读取失败",
    loading: "正在读取",
    missing: "项目路径不存在",
    ready: "已就绪",
  };
  return labels[status];
}

function projectsWithLiveStatus(
  projects: AgentFlowProjectRef[],
  activeProjectRoot: string | null,
  projectFilesState: ProjectFilesState,
  stateStatusState: StateStatusState,
) {
  return projects.map((project) =>
    project.root === activeProjectRoot
      ? {
          ...project,
          status: project.status === "missing" ? project.status : activeProjectStatus(projectFilesState, stateStatusState),
        }
      : project,
  );
}

function activeProjectStatus(
  projectFilesState: ProjectFilesState,
  stateStatusState: StateStatusState,
): AgentFlowProjectStatus {
  if (projectFilesState.error || stateStatusState.error) {
    return "error";
  }
  if (projectFilesState.source === "loading" || stateStatusState.source === "loading") {
    return "loading";
  }
  if (stateStatusState.status?.blockers.length) {
    return "blocked";
  }
  return "ready";
}

function displayRiskLabelZh(risk?: string | null) {
  return displayRiskTextZh(risk);
}

function issueCategoryLabelZh(category?: string | null) {
  if (category === "audit") {
    return "审计任务";
  }
  return "需求任务";
}

function agentRoleLabelZh(role?: string | null) {
  const labels: Record<string, string> = {
    "audit-agent": "审计助手",
    "build-agent": "执行助手",
    "spec-agent": "需求助手",
  };
  return labels[role ?? ""] ?? "执行助手";
}

function codexRoleGuideForRole(role?: string | null) {
  return codexRoleGuides.find((guide) => guide.role === role) ?? codexRoleGuides[1];
}

function codexThreadNameForRole(role?: string | null) {
  return codexRoleGuideForRole(role).threadName;
}

function agentRoleEnglishName(role?: string | null) {
  return codexRoleGuideForRole(role).englishName;
}

function agentInstructionForTask(task: V1Issue) {
  if (task.requiredAgentRole === "audit-agent") {
    return "你现在是 Audit Agent，只能执行 audit issue。如果你不是 audit-agent，请停止执行。不要修改源码、不要生成 patch、不要创建远程 PR。";
  }
  return "你现在是 Build Agent，只能执行 spec issue。如果你不是 build-agent，请停止执行。AgentFlow 当前 issue、handoff package 和 executionPipeline 是唯一任务源；不要把外部 issue、任务、计划、队列、线程或工具状态当成任务源，也不要用外部状态拆分、重排或推进 AgentFlow 任务。按 GitHub 自动化预检、执行、沙箱验证、创建 PR、合并 PR、写回 Done 的流程执行。写回 Done 前必须确认当前 AgentFlow CLI 支持 build-agent complete；不要直接复用过期 target/release/agentflow。不要写 audit report、findings、evidence-map 或 traceability。";
}

function taskAuditDescriptionItems(task: V1Issue): Array<[string, string]> {
  if (task.issueCategory !== "audit") {
    return [];
  }

  return [
    ["审计目标", task.auditId || "未提供"],
    ["关联 Release", task.sourceReleaseId || "未提供"],
    ["交付文件", task.sourceDeliveryPath || "未提供"],
    ["输出目录", task.auditOutputDir || "未提供"],
  ];
}

function taskOutputItems(task: V1Issue) {
  const entries = Object.entries(task.expectedOutputs ?? {});
  if (!entries.length) {
    return ["未提供输出位置。"];
  }
  return entries.map(([key, value]) => `${key}: ${value}`);
}

function projectRecommendedIssue(
  group: TaskProjectGroup,
  treeSelection: TaskProjectTreeViewModel["selection"] | null,
) {
  const selectedIssue =
    treeSelection?.kind === "issue"
      ? group.issues.find((issue) => issue.id === treeSelection.issueId) ?? null
      : null;
  if (selectedIssue) {
    return selectedIssue;
  }

  return (
    group.issues.find((issue) => issue.displayStatus === "in-progress") ??
    group.issues.find((issue) => issue.displayStatus === "ready") ??
    group.issues.find((issue) => issue.displayStatus === "review") ??
    group.issues.find((issue) => issue.displayStatus === "backlog") ??
    group.issues.find((issue) => issue.displayStatus !== "done" && issue.displayStatus !== "cancel") ??
    group.issues[0] ??
    null
  );
}

function projectProgressItems(group: TaskProjectGroup) {
  if (!group.issues.length) {
    return ["还没有任务。"];
  }

  const readyCount = group.issues.filter((issue) => issue.displayStatus === "ready").length;
  const reviewCount = group.issues.filter((issue) => issue.displayStatus === "review").length;
  const backlogCount = group.issues.filter((issue) => issue.displayStatus === "backlog").length;

  return [
    `总任务：${group.counts.issueCount}`,
    `进行中：${group.counts.activeIssueCount}`,
    `就绪：${readyCount}`,
    `待审阅：${reviewCount}`,
    `待办：${backlogCount}`,
    `已完成：${group.counts.doneIssueCount}`,
    `审计任务：${group.counts.auditIssueCount}`,
  ];
}

function projectRiskSummaryItems(group: TaskProjectGroup) {
  if (!group.issues.length) {
    return ["还没有任务风险记录。"];
  }

  const risk = groupRiskLevel(group.issues);
  const elevatedIssues = group.issues.filter((issue) => {
    const tone = riskToneKey(issue.riskLevel);
    return tone === "high" || tone === "medium";
  });

  return [
    `整体风险：${displayRiskTextZh(risk)}`,
    ...(elevatedIssues.length
      ? elevatedIssues.slice(0, 5).map((issue) => `${issue.id}：${displayRiskTextZh(issue.riskLevel)} · ${issue.title}`)
      : ["没有中高风险任务。"]),
  ];
}

function projectRecommendedIssueItems(issue: TaskIssueNode | null) {
  if (!issue) {
    return ["暂无建议任务。"];
  }

  return [
    `${issue.id}：${issue.title}`,
    `状态：${displayStatusLabelZh(issue.displayStatus)}`,
    `任务类型：${issueCategoryLabelZh(issue.issueCategory)}`,
    `执行角色：${agentRoleLabelZh(issue.requiredAgentRole)}`,
    issue.blockedBy.length ? `前置依赖：${issue.blockedBy.join("、")}` : "前置依赖：无",
  ];
}

function projectDependencySummaryItems(group: TaskProjectGroup) {
  if (!group.issues.length) {
    return ["还没有任务。"];
  }

  const details = group.issues.flatMap((issue) => [
    ...(issue.blockedBy.length ? [`${issue.id} 被 ${issue.blockedBy.join("、")} 阻塞。`] : []),
    ...(issue.blocks.length ? [`${issue.id} 阻塞 ${issue.blocks.join("、")}。`] : []),
  ]);
  const blockedByCount = group.issues.reduce((total, issue) => total + issue.blockedBy.length, 0);
  const blocksCount = group.issues.reduce((total, issue) => total + issue.blocks.length, 0);

  if (!details.length) {
    return ["没有记录依赖关系。"];
  }

  return [
    `前置依赖：${blockedByCount}`,
    `阻塞下游：${blocksCount}`,
    ...details.slice(0, 6),
    ...(details.length > 6 ? [`还有 ${details.length - 6} 条依赖未显示。`] : []),
  ];
}

function projectWarningItems(group: TaskProjectGroup) {
  return [
    ...group.missingIssueIds.map((issueId) => `缺失 issue 引用：${issueId}`),
    ...group.warnings.map((warning) => warning.message),
  ];
}

function taskExecutionPipeline(task: V1Issue) {
  if (task.issueCategory !== "spec") {
    return null;
  }
  return task.executionPipeline ?? defaultBuildAgentExecutionPipeline();
}

function taskExecutionPipelineItems(task: V1Issue) {
  const pipeline = taskExecutionPipeline(task);
  if (!pipeline?.stages.length) {
    return ["等待执行流程。"];
  }
  return pipeline.stages.map((stage, index) => `${index + 1}. ${stage.label}：${stage.goal}`);
}

function hasCompleteBuildAgentPipeline(task: V1Issue) {
  const pipeline = taskExecutionPipeline(task);
  if (!pipeline) {
    return false;
  }
  if (pipeline.version !== "build-agent-execution-pipeline.v1" || pipeline.agentRole !== "build-agent") {
    return false;
  }
  return buildAgentPipelineStageIds.every((stageId) =>
    pipeline.stages.some((stage) => stage.stageId === stageId && stage.required),
  );
}

function taskHandoffValidationError(task: V1Issue) {
  if (task.issueCategory === "audit") {
    if (!task.auditId || !task.auditOutputDir || !hasExpectedOutputs(task.expectedOutputs)) {
      return INCOMPLETE_HANDOFF_MESSAGE;
    }
    return null;
  }

  const outputs = task.expectedOutputs ?? {};
  if (
    !task.sourceSpecId ||
    !hasBuildExpectedOutputs(outputs) ||
    !hasCompleteBuildAgentPipeline(task)
  ) {
    return INCOMPLETE_HANDOFF_MESSAGE;
  }
  return null;
}

function hasExpectedOutputs(outputs?: ExpectedOutputs | null) {
  return Object.values(outputs ?? {}).some((value) => typeof value === "string" && value.trim().length > 0);
}

function hasBuildExpectedOutputs(outputs?: ExpectedOutputs | null) {
  return Boolean(
    outputs?.executeRunDir?.trim() &&
      outputs.evidencePath?.trim() &&
      outputs.releaseDeliveryDir?.trim(),
  );
}

function taskActionDisplayLabel(action: TaskInteractionAction, task: V1Issue, copyState: ButtonInteractionState) {
  if (action === "copy-handoff") {
    if (copyState === "success") {
      return "已复制";
    }
    return "复制任务";
  }
  return taskActionLabel(action);
}

function displayRiskTextZh(risk?: string | null) {
  const tone = riskToneKey(risk);
  if (tone === "high") {
    return "高";
  }
  if (tone === "medium") {
    return "中";
  }
  return "低";
}

function riskToneKey(risk?: string | null) {
  const normalized = (risk ?? "low").toLowerCase();
  if (normalized.includes("critical") || normalized.includes("high")) {
    return "high";
  }
  if (normalized.includes("medium") || normalized === "med") {
    return "medium";
  }
  if (normalized.includes("low")) {
    return "low";
  }
  return "low";
}

function groupRiskLevel(issues: TaskIssueNode[]) {
  const tones = issues.map((issue) => riskToneKey(issue.riskLevel));
  if (tones.includes("high")) {
    return "high";
  }
  if (tones.includes("medium")) {
    return "medium";
  }
  if (tones.includes("low")) {
    return "low";
  }
  return "low";
}

function riskStatusDotClass(risk?: string | null) {
  return `v16-risk-dot-${riskToneKey(risk)}`;
}

function riskTextClass(risk?: string | null) {
  return `v16-risk-text-${riskToneKey(risk)}`;
}

function findDeliveryForTask(deliveries: OutputIndexEntry[], taskId: string) {
  return sortOutputEntriesByLatest(deliveries)
    .find((delivery) => delivery.issueId === taskId || delivery.runId.includes(taskId)) ?? null;
}

function auditHasReport(audit: AuditIndexEntry | null | undefined) {
  return Boolean(audit && audit.status !== "requested" && audit.status !== "running");
}

function findAuditForDelivery(audits: AuditIndexEntry[], deliveryRunId: string) {
  return sortAuditsByLatest(audits)
    .find(
      (audit) =>
        audit.sourceRunId === deliveryRunId ||
        audit.sourceDeliveryId === deliveryRunId ||
        audit.auditId.includes(deliveryRunId),
    ) ?? null;
}

function deliveryAuditStatus(delivery: OutputIndexEntry | null, audit: AuditIndexEntry | null) {
  if (!delivery) {
    return {
      actionLabel: "等待交付",
      canOpenReport: false,
      detail: "还没有 Release Delivery。",
    };
  }
  if (!audit) {
    return {
      actionLabel: "暂无审计",
      canOpenReport: false,
      detail: "交付已生成，暂无审计请求。任务完成不会自动触发审计。",
    };
  }
  if (audit.status === "requested") {
    return {
      actionLabel: "等待 Agent 审计",
      canOpenReport: false,
      detail: `${auditTriggerLabel(audit.trigger)}已登记，等待 Agent 写入审计报告。`,
    };
  }
  if (audit.status === "running") {
    return {
      actionLabel: "审计中",
      canOpenReport: false,
      detail: `${auditTriggerLabel(audit.trigger)}正在进行。`,
    };
  }
  return {
    actionLabel: "查看审计报告",
    canOpenReport: true,
    detail: `审计状态：已完成。${auditTriggerLabel(audit.trigger)}，结果：${artifactStatusLabel(audit.status)}。`,
  };
}

function buildRecentActivities(
  workspaceData: WorkspaceDataState,
  outputBundle: OutputBundleState,
  initializationStatus: ProjectInitializationStatus | null,
  outputSummary?: NonNullable<OutputStatusState["status"]>["summary"],
) {
  const initializationItems = [
    ...(initializationStatus?.recentContext.slice(0, 2).map((context) => ({
      detail: context.summary,
      id: `init-context-${context.id}`,
      target: "tasks" as const,
      title: "已读取最近项目记录",
    })) ?? []),
    ...(initializationStatus?.demoDataCreated
      ? [
          {
            detail: `${initializationStatus.demoIssueCount} 个示例任务，${initializationStatus.demoDeliveryCount} 个示例交付，${initializationStatus.demoAuditCount} 个示例审计`,
            id: "init-demo-data",
            target: "tasks" as const,
            title: "示例流程已准备",
          },
        ]
      : []),
  ];
  const projectUpdates =
    workspaceData.workbench?.projectUpdates.slice(-2).map((update, index) => ({
      detail: update.title || update.path,
      id: `update-${index}-${update.path}`,
      target: "tasks" as const,
      title: "项目更新已记录",
    })) ?? [];
  const deliveryItems =
    sortOutputEntriesByLatest(outputBundle.outputIndex?.releaseDeliveries ?? []).slice(0, 2).map((delivery) => ({
      detail: `${delivery.issueId || "关联任务"} · ${artifactStatusLabel(delivery.status)}`,
      id: `delivery-${delivery.runId}`,
      target: "delivery" as const,
      title: "交付页面同步结构",
    })) ?? [];
  const auditItems =
    sortAuditsByLatest(outputBundle.auditIndex?.audits ?? []).slice(0, 2).map((audit) => ({
      detail: `${audit.auditId} · ${artifactStatusLabel(audit.status)}`,
      id: `audit-${audit.auditId}`,
      target: "audit" as const,
      title: "审计页面同步结构",
    })) ?? [];

  const items = [...initializationItems, ...projectUpdates, ...deliveryItems, ...auditItems];
  if (items.length) {
    return items.slice(0, 4);
  }

  return [
    {
      detail: `${outputSummary?.releaseDeliveries ?? 0} 个交付，${outputSummary?.audits ?? 0} 个审计`,
      id: "activity-output",
      target: "delivery" as const,
      title: "交付页面同步结构",
    },
    {
      detail: "任务合约和状态按钮已按状态收口。",
      id: "activity-task",
      target: "tasks" as const,
      title: "任务页面压缩完成",
    },
    {
      detail: "高级页只读展示状态文件。",
      id: "activity-advanced",
      target: "audit" as const,
      title: "高级页面清理",
    },
  ];
}

function buildNextStep(
  stateStatusState: StateStatusState,
  inputStatus: ReturnType<typeof useInputStatus>["status"],
  outputStatus: ReturnType<typeof useOutputStatus>,
  selectedTask: V1Issue | null,
): NextStepViewModel {
  const blocker = stateStatusState.status?.blockers.at(0);
  if (blocker) {
    return {
      action: "查看阻断原因",
      description: `还不能继续。原因是：${blocker.reason}`,
      reason: blocker.sourcePath ?? blocker.action,
      status: "warning",
      title: "当前有阻断",
    };
  }

  if ((outputStatus.status?.summary.releaseDeliveries ?? 0) > 0) {
    return {
      action: "查看交付",
      description: "任务已完成，交付材料已生成。审计是独立流程，不会自动触发。",
      reason: "任务完成和审计请求分开处理。",
      status: "ready",
      title: "交付已生成",
    };
  }

  if ((inputStatus?.summary.approvedSpecs ?? 0) === 0) {
    return {
      action: "继续整理规格",
      description: "还不能进入执行。原因是：这个需求还没有确认成规格。",
      reason: "Spec Agent 需要先整理需求，再由人确认。",
      status: "warning",
      title: "先确认需求",
    };
  }

  if (selectedTask) {
    return {
      action: "复制执行指令",
      description: "这个任务已经有已确认规格和任务合同。",
      reason: selectedTask.id,
      status: "ready",
      title: "可以进入执行了",
    };
  }

  return {
    action: "告诉 Agent 你想做什么",
    description: "AgentFlow 已经准备好规则和项目现场。",
    reason: "下一步从一个清楚的需求入口开始。",
    status: "ready",
    title: "项目已准备好",
  };
}

function buildCodexHandoff(task: V1Issue) {
  const handoffPackage =
    task.issueCategory === "audit"
      ? {
          agentInstruction: agentInstructionForTask(task),
          auditId: task.auditId,
          auditOutputDir: task.auditOutputDir,
          expectedOutputs: task.expectedOutputs,
          handoffId: task.handoffId,
          handoffVersion: "agent-handoff.v1",
          issueCategory: "audit",
          issueId: task.id,
          projectId: task.projectId,
          requiredAgentRole: task.requiredAgentRole ?? "audit-agent",
          sourceDeliveryPath: task.sourceDeliveryPath,
          sourceReleaseId: task.sourceReleaseId,
        }
      : {
          agentInstruction: agentInstructionForTask(task),
          completionWriteback: {
            cli: "target/release/agentflow build-agent complete --request <completion-request.json> after cargo build --release --bin agentflow, or target/debug/agentflow build-agent complete --request <completion-request.json>",
            command: "complete_build_agent_issue",
            request: {
              changedFiles: task.allowedFiles.slice(0, 3).map((path) => ({
                changeType: "modified",
                deletions: 0,
                insertions: 0,
                path,
              })),
              issueId: task.id,
              validationCommands: task.validationCommands.slice(0, 3).map((command) => ({
                args: command.split(" ").slice(1),
                exitCode: 0,
                label: command,
                program: command.split(" ")[0] ?? command,
                source: "build-agent",
              })),
            },
          },
          contextPackPath: task.contextPackPath,
          executionPipeline: taskExecutionPipeline(task),
          expectedOutputs: task.expectedOutputs,
          handoffId: task.handoffId,
          handoffVersion: "agent-handoff.v1",
          issueCategory: "spec",
          issueId: task.id,
          issuePath: task.issuePath,
          projectId: task.projectId,
          requiredAgentRole: task.requiredAgentRole ?? "build-agent",
          sourceSpecId: task.sourceSpecId,
          sourceSpecPath: task.sourceSpecPath,
        };
  return [
    `# ${task.title}`,
    "",
    "```json",
    JSON.stringify(
      handoffPackage,
      null,
      2,
    ),
    "```",
    "",
    `任务：${task.id}`,
    `任务类型：${issueCategoryLabelZh(task.issueCategory)}`,
    `执行角色：${agentRoleLabelZh(task.requiredAgentRole)}`,
    `风险：${displayRiskLabelZh(task.riskLevel)}`,
    `指令：${agentInstructionForTask(task)}`,
    ...(task.issueCategory === "audit"
      ? [
          `审计目标：${task.auditId ?? ""}`,
          `关联 Release：${task.sourceReleaseId ?? ""}`,
          `审计输出目录：${task.auditOutputDir ?? ""}`,
        ]
      : [
          `上下文包：${task.contextPackPath ?? ""}`,
        ]),
    "",
    "## 角色边界",
    "- 如果你不是 requiredAgentRole，请停止执行。",
    "- 如果 issueCategory 不属于你，请停止执行。",
    "- 不要执行其他 Agent 的任务。",
    "- AgentFlow 当前 issue、handoff package 和 executionPipeline 是唯一任务源。",
    "- 不要把外部 issue、任务、计划、队列、线程或工具状态当成任务源。",
    "- 不要用外部状态拆分、重排或推进 AgentFlow 任务。",
    "- GitHub 命令只允许用于当前 executionPipeline 里的 PR 阶段。",
    "- 不要越过任务边界。",
    "- 不要手写 `.agentflow/execute/**`、`.agentflow/output/evidence/**` 或 `.agentflow/output/release/**`。",
    ...(task.issueCategory === "spec"
      ? [
          "",
          "## Build Agent 执行流程",
          ...taskExecutionPipelineItems(task).map((item) => `- ${item}`),
          "",
          "## 合并规则",
          "- 创建 PR 前必须完成 GitHub 自动化预检和沙箱测试与结果验证。",
          "- 创建 PR 不是终点。Draft PR 只是中间产物，不能直接写回 Done。",
          "- mergeMode = manual-merge：把 PR 标记 ready 后等待人合并，合并前停止。",
          "- mergeMode = auto-merge-if-eligible：执行 `gh pr ready`，再执行 `gh pr merge --auto`，轮询 PR merged 状态。",
          "- PR 合并后才写回 Done。",
          "- 写回 Done 前必须确认当前 AgentFlow CLI 支持 `build-agent complete`。",
          "- 如果使用 `target/release/agentflow`，必须先运行 `cargo build --release --bin agentflow`；否则使用 `target/debug/agentflow`。",
          "- 不要直接复用可能过期的 `target/release/agentflow`。",
        ]
      : []),
    "",
    "## 范围",
    ...task.scope.map((item) => `- ${item}`),
    "",
    "## 非目标",
    ...task.nonGoals.map((item) => `- ${item}`),
    "",
    "## 允许路径",
    ...task.allowedFiles.map((item) => `- ${item}`),
    "",
    "## 禁止动作",
    ...task.forbiddenActions.map((item) => `- ${item}`),
    "",
    "## 禁止路径",
    ...task.forbiddenFiles.map((item) => `- ${item}`),
    "",
    "## 验证命令",
    ...task.validationCommands.map((item) => `- ${item}`),
    "",
    "## 交付要求",
    ...task.evidenceRequired.map((item) => `- ${item}`),
    "",
    "## 输出位置",
    ...taskOutputItems(task).map((item) => `- ${item}`),
    ...(task.issueCategory === "spec"
      ? [
          "",
          "## 完成写回",
          "- 完成代码任务并确认 PR 已合并后，调用已验证的新 CLI 写回。",
          "- 使用 `target/release/agentflow` 前必须先运行 `cargo build --release --bin agentflow`。",
          "- 如果 release binary 不支持 `build-agent complete`，使用 `target/debug/agentflow build-agent complete --request <completion-request.json>`。",
          "- 桌面端内部命令名为 `complete_build_agent_issue`。",
          "- request.issueId 必须等于当前任务 id。",
          "- request.changedFiles 填写实际修改文件。",
          "- request.validationCommands 填写已执行的验证命令和 exitCode。",
          "- AgentFlow 会自动生成规范 run、evidence、release，并把任务派生成已完成。",
        ]
      : []),
  ].join("\n");
}

function agentRoleRulesDocument() {
  return {
    version: "agent-role-usage-guide.v1",
    rule: "AgentFlow 不直接控制执行过程。用户需要按角色开 3 个独立线程，每个线程只做一种工作。",
    warning: "不要在一个执行线程里混用多个角色。",
    source: {
      rolesJson: ".agentflow/define/agent/roles.json",
      rootAgentEntry: "AGENTS.md",
      manual: ".agentflow/define/agent/Agentflow.md",
    },
    roles: codexRoleGuides.map((guide) => ({
      agentRole: guide.role,
      label: guide.title,
      englishName: guide.englishName,
      agentThreadName: guide.threadName,
      summary: guide.summary,
      cannotDo: guide.cannotDo,
    })),
    matrix: [
      {
        agentRole: "spec-agent",
        handlesIssueCategory: [],
        writes: [".agentflow/input/**"],
      },
      {
        agentRole: "build-agent",
        handlesIssueCategory: ["spec"],
        writes: [".agentflow/execute/**", ".agentflow/output/evidence/**", ".agentflow/output/release/**"],
      },
      {
        agentRole: "audit-agent",
        handlesIssueCategory: ["audit"],
        writes: [".agentflow/output/audit/**"],
      },
    ],
  };
}

function buildNextActionLabel(action: string) {
  const labels: Record<string, string> = {
    "start-new-input": "告诉 Agent 你想做什么",
  };
  return labels[action] ?? action;
}

function auditTriggerLabel(trigger?: string | null) {
  const labels: Record<string, string> = {
    "human-via-agent": "人类通过 Agent 触发",
    "release-auto": "交付关联审计",
  };
  return labels[trigger ?? ""] ?? "审计规则";
}

function artifactStatusLabel(status?: string | null) {
  const labels: Record<string, string> = {
    accepted: "已接受",
    approved: "已确认",
    audit: "待审计",
    audited: "已审计",
    blocked: "阻断",
    completed: "已完成",
    done: "已完成",
    failed: "失败",
    missing: "缺失",
    pass: "通过",
    passed: "通过",
    "passed-with-warnings": "通过，有警告",
    pending: "待处理",
    ready: "就绪",
    requested: "已请求",
    running: "审计中",
    review: "待审计",
    validated: "已验证",
    waiting: "等待",
  };
  if (!status) {
    return "未记录";
  }
  return labels[status.toLowerCase()] ?? status;
}

function deliveryDisplayId(runId: string) {
  const suffix = runId.match(/(\d+)$/)?.[1];
  return suffix ? `DEL-${suffix.padStart(3, "0").slice(-3)}` : `DEL-${runId.slice(-6)}`;
}

function humanizeKey(key: string) {
  const labels: Record<string, string> = {
    audit: "审计",
    delivery: "交付",
    evidence: "证据",
    issue: "任务",
    releaseDelivery: "交付记录",
    run: "执行",
    spec: "规格",
    traceability: "追溯",
  };
  return labels[key] ?? key.replace(/([A-Z])/g, " $1").trim();
}

function summarizeValue(value: unknown): string {
  if (Array.isArray(value)) {
    return value.length ? `${value.length} 条记录` : "暂无记录";
  }
  if (value && typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (!entries.length) {
      return "暂无记录";
    }
    return entries
      .slice(0, 3)
      .map(([key, item]) => `${humanizeKey(key)}：${typeof item === "string" ? item : Array.isArray(item) ? `${item.length} 条` : "已记录"}`)
      .join("；");
  }
  if (value === null || value === undefined || value === "") {
    return "未记录";
  }
  return String(value);
}

function formatTimestamp(timestamp: number) {
  if (!timestamp) {
    return "未记录";
  }
  return new Date(timestamp * 1000).toLocaleString("zh-CN", {
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    month: "2-digit",
  });
}

function pageTitle(page: AppPage) {
  const labels: Record<AppPage, string> = {
    advanced: "高级",
    audit: "审计",
    delivery: "交付",
    files: "文件",
    home: "工作台",
    tasks: "任务流转",
  };
  return labels[page] ?? "工作台";
}

function workflowStageText(stage?: string | null) {
  const labels: Record<string, string> = {
    "audit-completed": "审计完成",
    "audit-requested": "审计已请求",
    "delivery-ready": "交付可审计",
    "execute-ready": "可执行",
    "input-ready": "需求等待确认",
    "workspace-ready": "项目已准备好",
  };
  return stage ? labels[stage] ?? stage : "等待状态";
}

function titlebarStatusText(
  appInteractionState: AppInteractionState,
  stage: string | null | undefined,
  selectedTask: V1Issue | null,
) {
  if (appInteractionState.lifecycle === "not-authenticated") {
    return "not-authenticated";
  }
  if (appInteractionState.lifecycle === "first-run") {
    return "first-run";
  }
  if (appInteractionState.lifecycle === "project-loading") {
    return "loading";
  }
  if (appInteractionState.lifecycle === "error") {
    return "error";
  }
  if (appInteractionState.lifecycle === "workspace-blocked") {
    return "blocked";
  }

  if (selectedTask?.displayStatus === "ready") {
    return "waiting-for-agent";
  }
  if (selectedTask?.displayStatus === "in-progress") {
    return "agent-running";
  }
  if (selectedTask?.displayStatus === "review") {
    return "ready-for-audit";
  }
  if (selectedTask?.displayStatus === "done") {
    return "delivered";
  }

  const labels: Record<string, string> = {
    "audit-completed": "audit-completed",
    "audit-requested": "audit-requested",
    "delivery-ready": "ready-for-audit",
    "execute-ready": "waiting-for-agent",
    "input-ready": "needs-spec",
    "workspace-ready": "workspace-ready",
  };

  return stage ? labels[stage] ?? stage : "workspace-ready";
}

function advancedCategorySummary(categoryId: string) {
  const summaries: Record<string, string> = {
    agentRoles: "展示三个执行线程的角色边界和 roles.json 只读诊断规则。",
    audit: "展示审计索引和报告快照。这里不写处理结果。",
    execute: "展示执行状态快照。这里不继续执行，不清理锁。",
    initialization: "展示基础发布初始化摘要。这里不重跑初始化，不删除示例数据。",
    input: "展示需求和 Issue 状态快照。普通页面只展示人能读懂的摘要。",
    output: "展示证据、交付和审计输出摘要。",
    panel: "展示项目现场读取结果和上下文包摘要。",
    settings: "展示本地设置、文件阅读器和工作台数据源状态。",
    state: "展示全局派生状态、门禁、阻断和下一步动作。",
  };
  return summaries[categoryId] ?? "这里展示开发者调试信息。普通页面不显示原始 JSON。";
}

function advancedFilesForCategory(categoryId: string) {
  const files: Record<string, Array<{ description: string; name: string }>> = {
    agentRoles: [
      { name: ".agentflow/define/agent/roles.json", description: "三类 Agent 的可处理任务和写入边界" },
      { name: "AGENTS.md", description: "根级 Agent 入口规则" },
      { name: ".agentflow/define/agent/Agentflow.md", description: "AgentFlow 工作手册" },
    ],
    audit: [
      { name: "index.json", description: "审计报告索引" },
      { name: "audit.json", description: "审计结论和检查结果" },
      { name: "evidence-map.json", description: "证据链映射" },
      { name: "traceability.json", description: "规格、任务和交付追溯" },
    ],
    execute: [
      { name: "runs/index.json", description: "执行运行列表" },
      { name: "leases/*.json", description: "本地执行锁状态" },
      { name: "commands/*.json", description: "命令记录" },
    ],
    initialization: [
      { name: "base-release-initialization.json", description: "基础发布初始化摘要" },
      { name: "recent-project-context.json", description: "现有项目最近提交上下文" },
      { name: "git-context.json", description: "本地 Git 上下文索引" },
    ],
    input: [
      { name: "index.json", description: "规格、项目和任务索引" },
      { name: "issues/*.json", description: "任务合约来源" },
      { name: "specs/approved/*", description: "已确认规格" },
    ],
    output: [
      { name: "index.json", description: "证据、交付和审计输出索引" },
      { name: "evidence/*.json", description: "验证证据" },
      { name: "release/*/delivery.json", description: "交付包记录" },
    ],
    panel: [
      { name: "manifest.json", description: "项目现场摘要" },
      { name: "context-packs/*.json", description: "上下文包" },
      { name: "diagnostics.json", description: "诊断快照" },
    ],
    settings: [
      { name: "locale.json", description: "Agent 语言设置" },
      { name: "style.json", description: "输出风格策略" },
      { name: "AGENTS.md", description: "本地 Agent 入口文件" },
    ],
    state: [
      { name: "workflow.json", description: "当前阶段与下一动作" },
      { name: "gates.json", description: "门禁检查结果" },
      { name: "blockers.json", description: "阻塞项快照" },
      { name: "locks.json", description: "本地锁状态" },
      { name: "sessions.json", description: "智能体会话记录" },
      { name: "next-actions.json", description: "下一步候选动作" },
    ],
  };
  return files[categoryId] ?? files.state;
}

function lifecycleLabel(state: AppInteractionState["lifecycle"]) {
  const labels: Record<AppInteractionState["lifecycle"], string> = {
    error: "错误",
    "first-run": "首次引导",
    "not-authenticated": "未连接",
    "project-loading": "项目加载中",
    "workspace-blocked": "工作区阻断",
    "workspace-ready": "工作区就绪",
  };
  return labels[state];
}

export default App;
