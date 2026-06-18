import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  CheckCircle2,
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
} from "./browserPreviewData";
import {
  ActionButton,
  AppFrame,
  CopyableCodeBlock,
  MetricCard,
  PageHeader,
  Panel,
  PriorityBadge,
  Section,
  Sidebar,
  StatusBadge,
  StatusBar as FoundationStatusBar,
  TopBar,
  WindowChrome,
  type StatusChipStatus,
} from "./components";
import { useAgentManual } from "./features/agent-manual";
import { useInputSnapshot, type InputSnapshotState } from "./features/input";
import { useMcpSessions, type McpSessionsState } from "./features/mcp";
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
import {
  useIssueStatusIndex,
  useProjectProjection,
  useStateStatus,
  useTaskProjection,
  type IssueStatusIndexState,
  type StateStatusState,
} from "./features/state";
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
  buildTaskDeliveryProjection,
  buildTaskExecutionProjection,
  buildTaskStatusContract,
  buildTaskStatusTimeline,
  buildTaskInteractionState,
  buildTaskProjectTreeViewModel,
  buildTaskWorkflowYamlModel,
  displayStatusLabelZh,
  pickTaskId,
  sortTasksByExecutionOrder,
  taskActionLabel,
  type AppInteractionState,
  type ButtonInteractionState,
  type TaskDeliveryProjection,
  type TaskExecutionProjection,
  type TaskInteractionAction,
  type TaskIssueNode,
  type TaskProjectGroup,
  type TaskProjectTreeViewModel,
  type TaskWorkflowYamlModel,
} from "./interaction/viewModels";
import type {
  AuditIndex,
  AuditIndexEntry,
  ExecuteStatusSnapshot,
  ExecutionPipeline,
  HumanAuditReport,
  McpLogChunk,
  McpSessionSnapshot,
  AgentRole,
  InputIssue,
  IssueDisplayStatus,
  IssueStatusIndex,
  OutputIndex,
  OutputIndexEntry,
  ProjectProjection,
  ProjectionPhase,
  StateStatusSnapshot,
  TaskProjection,
  TaskTimelineItem,
  V1Issue,
  ExpectedOutputs,
} from "./types";
import "./AppShell.css";

type Provider = "ChatGPT" | "Claude" | "DeepSeek";
type AppPage = AgentFlowProjectPage;
type DataSource = "idle" | "loading" | "tauri" | "preview" | "unavailable";

type WorkspaceDataState = {
  error: string | null;
  source: DataSource;
};

type OutputBundleState = {
  auditIndex: AuditIndex | null;
  auditReport: HumanAuditReport | null;
  deliveryArtifacts: Record<string, DeliveryArtifactState>;
  error: string | null;
  outputIndex: OutputIndex | null;
  source: DataSource;
};

type ExecuteStatusState = {
  status: ExecuteStatusSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

type DeliveryPrMetadataState = {
  branchName?: string | null;
  createdRemotePr: boolean;
  mergeMode?: string | null;
  merged: boolean;
  provider?: string | null;
  remotePrUrl?: string | null;
  status?: string | null;
  title?: string | null;
};

type DeliveryMergeProofState = {
  mergeMode?: string | null;
  merged: boolean;
  provider?: string | null;
  remoteUrl?: string | null;
};

type DeliveryReleaseNoteState = {
  summaryLines: string[];
  title?: string | null;
};

type DeliveryArtifactState = {
  mergeProof: DeliveryMergeProofState | null;
  prMetadata: DeliveryPrMetadataState | null;
  releaseNote: DeliveryReleaseNoteState | null;
  runId: string;
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

type WorkflowEventsDispatchedEvent = {
  buildAgentLaunchSessionsCreated: number;
  contextPackFailed: number;
  contextPackReady: number;
  contextPackRequests: number;
  errors: string[];
  pendingBuildAgentLaunchEvents: number;
  pendingPanelEvents: number;
  projectRoot: string;
  version: string;
};

type ProjectLoopTickedEvent = {
  activeIssueCount: number;
  blockedIssueCount: number;
  directIssueCount: number;
  doneIssueCount: number;
  errors: string[];
  projectCount: number;
  projectRoot: string;
  runtimeLaunchCount: number;
  version: string;
};

const AGENTFLOW_WORKSPACE_CHANGED_EVENT = "agentflow-workspace-changed";
const AGENTFLOW_WORKFLOW_EVENTS_DISPATCHED_EVENT = "agentflow-task-events-dispatched";
const AGENTFLOW_PROJECT_LOOP_TICKED_EVENT = "agentflow-project-loop-ticked";
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
  { icon: ShieldCheck, id: "audit", label: "审计" },
  { icon: FileSearch, id: "files", label: "文件" },
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
const INCOMPLETE_HANDOFF_MESSAGE = "这个任务包不完整，缺少执行目标。请先修复任务元数据。";

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
      "3. 生成任务。",
      "",
      "你不能做：",
      "- 不改代码",
      "- 不执行命令",
      "- 不生成 release",
      "- 不写 audit report",
      "- 不执行 Build Agent 或 Audit Agent 的任务",
      "",
      "你必须遵守：",
      "- 先写 docs/requirements/**，确认后再写 .agentflow/spec/**",
      "- 不修改用户源码",
      "- 不写 .agentflow/tasks/**",
      "- 不写 .agentflow/audit/**",
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
      "AgentFlow 当前 spec issue 是唯一任务源。",
      "handoff package 只是这份 issue 的派生快照。",
      "executionPipeline 只是这份 issue 合同里的一部分，不是独立任务源。",
      "不要把外部 issue、任务、计划、队列、线程或工具状态当成任务源。",
      "不要用外部状态拆分、重排或推进 AgentFlow 任务。",
      "GitHub/GitLab 命令只允许用于当前 executionPipeline 里的 PR/MR 阶段。",
      "",
      "你要做：",
      "1. 执行前置检测。",
      "2. 测试设计。",
      "3. Agent 执行 issue。",
      "4. 沙箱验证。",
      "5. 创建 PR/MR。",
      "6. 合并 PR/MR。",
      "7. 写回 Done。",
      "",
      "你不能做：",
      "- 不执行 audit issue",
      "- 不写 audit report",
      "- 不写 findings.json",
      "- 不写 evidence-map.json",
      "- 不写 traceability.json",
      "- 不越过任务边界",
      "- 不绕过执行前置检测",
      "- 不绕过测试设计",
      "- 不绕过沙箱验证",
      "- 不越过 mergeMode 合并 PR/MR",
      "- 不把外部 issue / task / plan / queue 当成任务源",
      "- 不用外部状态拆分、重排或推进 AgentFlow 任务",
      "- 不 deploy",
      "",
      "创建 PR/MR 前必须完成执行前置检测、测试设计和沙箱验证。",
      "合并 PR/MR 默认先走 auto-merge-if-eligible，manual-merge 只作为 fallback。",
      "如果 mergeMode = auto-merge-if-eligible，不能停在 Draft PR/MR；GitHub 执行 gh pr ready、gh pr merge --auto；GitLab 执行 glab mr update --ready、glab mr merge --auto-merge；然后轮询 PR/MR 是否 merged。",
      "如果自动合并条件不满足，回落到 manual-merge：PR/MR ready 后 issue 保持 in_review，等待人合并；本地检测确认 PR/MR merged 后才能写回 Done。",
      "写回 Done 前必须确认当前 AgentFlow CLI 支持 build-agent complete。",
      "如果使用 target/release/agentflow，必须先运行 cargo build --release --bin agentflow；否则使用 target/debug/agentflow。",
      "不要直接复用可能过期的 target/release/agentflow。",
      "进入 in_progress 前必须确认 Context Pack 可读，且当前工作区没有未提交的用户源码改动。",
      "如果任务不是 spec issue，必须停止。",
      "如果 requiredAgentRole 不是 build-agent，必须停止。",
    ].join("\n"),
    summary: "任务打包 · 执行改动 · 写回结果",
    threadName: "AgentFlow / Build Agent",
    title: "执行助手",
  },
  {
    cannotDo: ["不改代码", "不执行 spec issue", "不生成 release", "不创建 PR/MR / merge / deploy"],
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
      "1. 读取审计任务。",
      "2. 读取关联 SPEC / 任务 / Evidence / Release。",
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
      "- 不创建 PR/MR",
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
  if (value === "execute" || value === "delivery") {
    return "tasks";
  }
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
  const [taskDetailFocus, setTaskDetailFocus] = useState<"delivery" | null>(null);
  const [selectedAuditId, setSelectedAuditId] = useState<string | null>(null);
  const [taskListRefreshToken, setTaskListRefreshToken] = useState(0);
  const [executeRefreshToken, setExecuteRefreshToken] = useState(0);
  const [mcpRefreshToken, setMcpRefreshToken] = useState(0);
  const [outputRefreshToken, setOutputRefreshToken] = useState(0);
  const [stateRefreshToken, setStateRefreshToken] = useState(0);
  const [selectedIntent, setSelectedIntent] = useState("我要新增功能");
  const [onboardingFeedback, setOnboardingFeedback] = useState<string | null>(null);
  const [taskActionFeedback, setTaskActionFeedback] = useState<string | null>(null);
  const [taskCopyState, setTaskCopyState] = useState<ButtonInteractionState>("enabled");
  const [projectLoopState, setProjectLoopState] = useState<ButtonInteractionState>("enabled");
  const [projectLoopFeedback, setProjectLoopFeedback] = useState<string | null>(null);
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
  const inputSnapshotState = useInputSnapshot(projectRoot, taskListRefreshToken);
  const executeStatusState = useMemo<ExecuteStatusState>(
    () => ({ error: null, source: "idle", status: null }),
    [],
  );
  const mcpSessionsState = useMcpSessions(projectRoot, mcpRefreshToken);
  const stateStatusState = useStateStatus(projectRoot, stateRefreshToken);
  const issueStatusIndexState = useIssueStatusIndex(
    projectRoot,
    taskListRefreshToken + executeRefreshToken + mcpRefreshToken + outputRefreshToken,
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
    setTaskDetailFocus(null);
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
      const refreshSpec = refreshAll || changedAreas.has("spec");
      const refreshProjection = refreshAll || changedAreas.has("projections") || changedAreas.has("indexes");
      const refreshTasks = refreshAll || changedAreas.has("tasks");
      const refreshEvents = refreshAll || changedAreas.has("events");
      const refreshExecute = refreshAll || changedAreas.has("execute");
      const refreshOutput = refreshAll || changedAreas.has("output");
      const refreshPanel = refreshAll || changedAreas.has("panel");
      const refreshTaskList =
        refreshSpec || refreshProjection || refreshTasks || refreshEvents || refreshExecute || refreshOutput || refreshAll;

      if (refreshTaskList) {
        setTaskListRefreshToken((current) => current + 1);
        setStateRefreshToken((current) => current + 1);
      }

      if (refreshExecute && (activePage === "tasks" || activePage === "advanced")) {
        setExecuteRefreshToken((current) => current + 1);
        setMcpRefreshToken((current) => current + 1);
      }

      if (refreshOutput && (activePage === "tasks" || activePage === "advanced")) {
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
    if (!projectRoot || isBrowserPreviewRuntime()) {
      return;
    }

    let cancelled = false;
    let unlisten: (() => void) | null = null;
    void listen<WorkflowEventsDispatchedEvent>(AGENTFLOW_WORKFLOW_EVENTS_DISPATCHED_EVENT, (event) => {
      if (cancelled) {
        return;
      }
      const payload = event.payload;
      if (normalizeProjectRootKey(payload.projectRoot) !== normalizeProjectRootKey(projectRoot)) {
        return;
      }
      setTaskListRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
      if (payload.buildAgentLaunchSessionsCreated > 0) {
        setMcpRefreshToken((current) => current + 1);
        setExecuteRefreshToken((current) => current + 1);
      }
    }).then((cleanup) => {
      if (cancelled) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [projectRoot]);

  useEffect(() => {
    if (!projectRoot || isBrowserPreviewRuntime()) {
      return;
    }

    let cancelled = false;
    let unlisten: (() => void) | null = null;
    void listen<ProjectLoopTickedEvent>(AGENTFLOW_PROJECT_LOOP_TICKED_EVENT, (event) => {
      if (cancelled) {
        return;
      }
      const payload = event.payload;
      if (normalizeProjectRootKey(payload.projectRoot) !== normalizeProjectRootKey(projectRoot)) {
        return;
      }
      setTaskListRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
      if (payload.runtimeLaunchCount > 0) {
        setMcpRefreshToken((current) => current + 1);
        setExecuteRefreshToken((current) => current + 1);
        void invoke("dispatch_workflow_events", { projectRoot }).catch(() => undefined);
      }
    }).then((cleanup) => {
      if (cancelled) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [projectRoot]);

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
        setTaskListRefreshToken((current) => current + 1);
        setExecuteRefreshToken((current) => current + 1);
        setMcpRefreshToken((current) => current + 1);
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
    () => buildTaskItems(inputSnapshotState.snapshot ? inputSnapshotState.snapshot.issues : null, issueStatusIndexState.index),
    [inputSnapshotState.snapshot, issueStatusIndexState.index],
  );

  const filteredTasks = useMemo(() => {
    const query = taskSearch.trim().toLowerCase();
    if (!query) {
      return tasks;
    }
    return tasks.filter((task) => {
      const searchable = [task.id, task.title, task.displayStatus, task.status, task.priority, task.goal]
        .join(" ")
        .toLowerCase();
      return searchable.includes(query);
    });
  }, [taskSearch, tasks]);
  const taskProjectTree = useMemo(
    () =>
      inputSnapshotState.snapshot
        ? buildTaskProjectTreeViewModel({
            activeIssueId: null,
            issueStatusIndex: issueStatusIndexState.index,
            issues: inputSnapshotState.snapshot.issues,
            projects: inputSnapshotState.snapshot.projects,
            relations: inputSnapshotState.snapshot.relations,
          })
        : null,
    [inputSnapshotState.snapshot, issueStatusIndexState.index],
  );
  const selectedProjectGroup = useMemo(
    () => taskProjectTree?.groups.find((group) => group.id === selectedTaskProjectId) ?? null,
    [selectedTaskProjectId, taskProjectTree],
  );
  const selectedTaskCandidateId = useMemo(
    () => (selectedProjectGroup ? null : pickTaskId(tasks, selectedTaskId, null)),
    [selectedProjectGroup, selectedTaskId, tasks],
  );
  const taskInteractionState = useMemo(
    () => buildTaskInteractionState(tasks, selectedTaskCandidateId),
    [selectedTaskCandidateId, tasks],
  );
  const selectedTask = taskInteractionState.selectedTask;
  const taskProjectionState = useTaskProjection(
    projectRoot,
    selectedTask?.id ?? null,
    taskListRefreshToken + executeRefreshToken + mcpRefreshToken + outputRefreshToken,
  );
  const projectProjectionState = useProjectProjection(
    projectRoot,
    selectedProjectGroup?.id ?? null,
    taskListRefreshToken + executeRefreshToken + mcpRefreshToken + outputRefreshToken,
  );
  const selectedTaskProjection =
    taskProjectionState.projection?.issueId === selectedTask?.id ? taskProjectionState.projection : null;
  const selectedProjectProjection =
    projectProjectionState.projection?.projectId === selectedProjectGroup?.id
      ? projectProjectionState.projection
      : null;
  const agentLocale = agentManualState.status?.locale.agentLocale ?? detectAppLocale() ?? "en-US";
  const nextStep = useMemo(
    () => buildNextStep(stateStatusState, inputSnapshotState.snapshot?.issues.length ?? 0, selectedTask),
    [inputSnapshotState.snapshot?.issues.length, selectedTask, stateStatusState],
  );
  const activeProjectLiveStatus = projectRoot
    ? activeProjectRegistryStatus === "missing"
      ? "missing"
      : activeProjectStatus(projectFilesState, stateStatusState)
    : null;
  const appInteractionState: AppInteractionState = useMemo(
    () => {
      const outputPageHasError = (activePage === "tasks" || activePage === "audit") && Boolean(outputBundle.error);
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
        workspaceBlocked: stateWorkspaceBlocked(stateStatusState.status),
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
      stateStatusState.status?.currentStage,
      stateStatusState.status?.status,
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

    const nextTaskId = pickTaskId(tasks, selectedTaskId, null);
    if (nextTaskId !== selectedTaskId) {
      setSelectedTaskId(nextTaskId);
    }
  }, [selectedTaskId, selectedTaskProjectId, taskProjectTree, tasks]);

  function refreshProjectPage(
    page: AppPage,
    root = projectRoot,
    options: { triggerProjectLoop?: boolean } = {},
  ) {
    if (!root) {
      return;
    }
    const triggerProjectLoop = options.triggerProjectLoop ?? false;

    if (page === "home") {
      void prepareProjectPanel(root);
      setTaskListRefreshToken((current) => current + 1);
      setMcpRefreshToken((current) => current + 1);
      setOutputRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
      return;
    }

    if (page === "tasks") {
      setTaskListRefreshToken((current) => current + 1);
      setExecuteRefreshToken((current) => current + 1);
      setMcpRefreshToken((current) => current + 1);
      setOutputRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
      if (triggerProjectLoop) {
        void invoke("run_project_loop", { projectRoot: root })
          .then(() => {
            setStateRefreshToken((current) => current + 1);
            setTaskListRefreshToken((current) => current + 1);
            setMcpRefreshToken((current) => current + 1);
            setExecuteRefreshToken((current) => current + 1);
            setOutputRefreshToken((current) => current + 1);
          })
          .catch(() => undefined);
      }
      return;
    }

    if (page === "files") {
      void loadProjectFiles(root);
      return;
    }

    if (page === "audit") {
      setOutputRefreshToken((current) => current + 1);
      return;
    }

    if (page === "advanced") {
      void loadProjectFiles(root);
      void loadAgentManual(root);
      void prepareProjectPanel(root);
      setTaskListRefreshToken((current) => current + 1);
      setExecuteRefreshToken((current) => current + 1);
      setMcpRefreshToken((current) => current + 1);
      setOutputRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
    }
  }

  function setActivePage(page: AppPage) {
    const shouldRefresh = page !== activePage;
    setProjectRegistry((current) => setProjectPage(current, current.activeProjectRoot, page));
    if (shouldRefresh) {
      refreshProjectPage(page, projectRoot, { triggerProjectLoop: false });
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
      refreshProjectPage(page, projectRootToSelect, { triggerProjectLoop: false });
    }
  }

  function handleSelectTask(taskId: string) {
    setSelectedTaskProjectId(null);
    setSelectedTaskId(taskId);
    setTaskActionFeedback(null);
    setTaskDetailFocus(null);
  }

  function handleSelectTaskProject(projectId: string) {
    setSelectedTaskId(null);
    setSelectedTaskProjectId(projectId);
    setTaskActionFeedback(null);
    setTaskDetailFocus(null);
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
    refreshProjectPage(activePage, projectRoot, { triggerProjectLoop: false });
  }

  async function handleRunProjectLoop() {
    if (!projectRoot || isBrowserPreviewRuntime()) {
      return;
    }

    setProjectLoopState("loading");
    setProjectLoopFeedback(null);
    try {
      const summary = await invoke<ProjectLoopTickedEvent>("run_project_loop", { projectRoot });
      setTaskListRefreshToken((current) => current + 1);
      setExecuteRefreshToken((current) => current + 1);
      setMcpRefreshToken((current) => current + 1);
      setStateRefreshToken((current) => current + 1);
      setProjectLoopState("success");
      setProjectLoopFeedback(
        summary.runtimeLaunchCount > 0
          ? `已触发 Project Loop，拉起 ${summary.runtimeLaunchCount} 条执行。`
          : summary.activeIssueCount > 0
            ? `Project Loop 已运行，当前有 ${summary.activeIssueCount} 条进行中任务。`
            : summary.blockedIssueCount > 0
              ? `Project Loop 已运行，当前有 ${summary.blockedIssueCount} 条阻断任务。`
              : "Project Loop 已运行，当前没有可推进任务。",
      );
      window.setTimeout(() => setProjectLoopState("enabled"), 1200);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setProjectLoopState("error");
      setProjectLoopFeedback(`运行失败：${message}`);
      window.setTimeout(() => setProjectLoopState("enabled"), 1600);
    }
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
        await navigator.clipboard.writeText(buildCodexHandoff(task, agentLocale));
        setTaskCopyState("success");
        setTaskActionFeedback(`已复制。请粘贴到 ${codexThreadNameForRole(task.requiredAgentRole)} 线程。`);
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
      if (["in_review", "done"].includes(task.displayStatus ?? "backlog") || task.latestRunId) {
        setSelectedTaskId(task.id ?? null);
        setTaskDetailFocus("delivery");
        setActivePage("tasks");
        setTaskActionFeedback("已刷新任务状态。交付摘要和最终记录在当前任务详情中查看。");
      } else {
        setTaskActionFeedback("还没有检测到写回结果。");
      }
      return;
    }

    if (action === "view-delivery") {
      if (["in_review", "done"].includes(task.displayStatus ?? "backlog") || task.latestRunId) {
        setSelectedTaskId(task.id ?? null);
        setTaskDetailFocus("delivery");
        setActivePage("tasks");
        setTaskActionFeedback("已定位交付信息。交付摘要和最终交付卡在当前任务详情中查看。");
      } else {
        setTaskActionFeedback("还没有交付结果。写回后会显示在当前任务详情中。");
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
            onRunProjectLoop={() => void handleRunProjectLoop()}
            nextStep={nextStep}
            onOpenAudit={() => setActivePage("audit")}
            onOpenTasks={() => setActivePage("tasks")}
            outputBundle={outputBundle}
            projectLoopFeedback={projectLoopFeedback}
            projectLoopState={projectLoopState}
            selectedTask={selectedTask}
            initializationState={initializationState}
          />
        ) : null}
        {projectRoot && !projectAvailabilityStatus && activePage === "tasks" ? (
          <TasksPage
            actionFeedback={taskActionFeedback}
            actions={taskInteractionState.actions}
            agentLocale={agentLocale}
            copyState={taskCopyState}
            detailFocus={taskDetailFocus}
            executeStatusState={executeStatusState}
            mcpSessionsState={mcpSessionsState}
            onDetailFocusHandled={() => setTaskDetailFocus(null)}
            onTaskAction={(action, task) => void handleTaskAction(action, task)}
            onSelectProjectGroup={handleSelectTaskProject}
            onSelectTask={handleSelectTask}
            outputBundle={outputBundle}
            projectRoot={projectRoot}
            selectedProjectGroup={selectedProjectGroup}
            selectedProjectProjection={selectedProjectProjection}
            selectedTask={selectedTask}
            selectedTaskProjection={selectedTaskProjection}
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
            inputSnapshotState={inputSnapshotState}
            issueStatusIndexState={issueStatusIndexState}
            outputBundle={outputBundle}
            projectRoot={projectRoot}
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
    source: "idle",
  });

  useEffect(() => {
    if (!projectRoot) {
      setState({ error: null, source: "idle" });
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setState({
        error: null,
        source: "preview",
      });
      return;
    }

    setState({ error: null, source: "idle" });
  }, [projectRoot]);

  return state;
}

function useOutputBundle(projectRoot: string | null, refreshToken: number): OutputBundleState {
  const [state, setState] = useState<OutputBundleState>({
    auditIndex: null,
    auditReport: null,
    deliveryArtifacts: {},
    error: null,
    outputIndex: null,
    source: "idle",
  });

  useEffect(() => {
    if (!projectRoot) {
      setState({
        auditIndex: null,
        auditReport: null,
        deliveryArtifacts: {},
        error: null,
        outputIndex: null,
        source: "idle",
      });
      return;
    }

    if (isBrowserPreviewRuntime()) {
      const outputIndex = createBrowserPreviewOutputIndex();
      setState({
        auditIndex: createBrowserPreviewAuditIndex(),
        auditReport: createBrowserPreviewHumanAuditReport(),
        deliveryArtifacts: createBrowserPreviewDeliveryArtifacts(),
        error: null,
        outputIndex,
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
    void invoke<AuditIndex>("load_audit_index", { projectRoot })
      .then(async (auditIndex) => {
        const latestAuditWithReport = sortAuditsByLatest(auditIndex.audits).find((audit) =>
          auditHasReport(audit),
        );
        const auditReport = latestAuditWithReport
          ? await invoke<HumanAuditReport>("load_audit_report", { auditId: latestAuditWithReport.auditId, projectRoot })
          : null;

        if (!cancelled) {
          setState({
            auditIndex,
            auditReport,
            deliveryArtifacts: {},
            error: null,
            outputIndex: null,
            source: "tauri",
          });
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
                  deliveryArtifacts: {},
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

function createBrowserPreviewDeliveryArtifacts(): Record<string, DeliveryArtifactState> {
  const previewArtifacts: Array<{ issueId: string; merged: boolean; runId: string }> = [
    { issueId: "iss-review", merged: false, runId: "run-browser-preview-002" },
    { issueId: "iss-done", merged: true, runId: "run-browser-preview-003" },
  ];
  return Object.fromEntries(
    previewArtifacts.map(({ issueId, merged, runId }) => [
      runId,
      {
        mergeProof: {
          mergeMode: merged ? "auto-merge-if-eligible" : "manual-merge",
          merged,
          provider: "github",
          remoteUrl: `https://github.com/atxinbao/agentflow/pull/${merged ? 101 : 102}`,
        },
        prMetadata: {
          branchName: `agentflow/browser-preview/${issueId}`,
          createdRemotePr: true,
          mergeMode: merged ? "auto-merge-if-eligible" : "manual-merge",
          merged,
          provider: "github",
          remotePrUrl: `https://github.com/atxinbao/agentflow/pull/${merged ? 101 : 102}`,
          status: merged ? "merged" : "open",
          title: `Preview ${issueId}`,
        },
        releaseNote: {
          summaryLines: ["公开交付记录已准备完成。", "这里只展示浏览器预览用的模拟交付摘要。"],
          title: "公开交付记录",
        },
        runId,
      },
    ]),
  );
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
          <p>添加一个本地项目后，AgentFlow 会准备任务、审计和文件工作区，并在任务页串起执行与交付信息。</p>
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
  nextStep,
  onOpenAudit,
  onRunProjectLoop,
  onOpenTasks,
  outputBundle,
  initializationState,
  projectLoopFeedback,
  projectLoopState,
  selectedTask,
}: {
  nextStep: NextStepViewModel;
  onOpenAudit: () => void;
  onRunProjectLoop: () => void;
  onOpenTasks: () => void;
  outputBundle: OutputBundleState;
  initializationState: ProjectInitializationState;
  projectLoopFeedback: string | null;
  projectLoopState: ButtonInteractionState;
  selectedTask: V1Issue | null;
}) {
  const recentActivities = buildRecentActivities(outputBundle, initializationState.status);

  return (
    <section className="v16-page v16-home-page" data-agentflow-page="workbench">
      <section className="v16-home-columns" aria-label="工作台总览">
        <Panel className="v16-home-column v16-home-task-column" title="当前任务">
          {selectedTask ? (
            <button className="v16-current-task-card" onClick={onOpenTasks} type="button">
              <span className="v16-current-task-meta">
                <span>{selectedTask.id}</span>
                <PriorityBadge priority={selectedTask.priority} />
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
          {nextStep.description ? <p className="v16-home-next-step">{nextStep.description}</p> : null}
          <ActionBar>
            <ActionButton
              loading={projectLoopState === "loading"}
              onClick={onRunProjectLoop}
              variant="secondary"
            >
              运行 Project Loop
            </ActionButton>
          </ActionBar>
          {projectLoopFeedback ? <p className="v16-feedback">{projectLoopFeedback}</p> : null}
        </Panel>

        <Panel className="v16-home-column" title="最近活动">
          <div className="v16-activity-list">
            {recentActivities.map((activity) => (
              <button
                key={activity.id}
                onClick={activity.target === "audit" ? onOpenAudit : onOpenTasks}
                type="button"
              >
                <strong>{activity.title}</strong>
                <span>{activity.detail}</span>
              </button>
            ))}
          </div>
        </Panel>
      </section>
      <CodexRoleGuideCard />
    </section>
  );
}

function CodexRoleGuideCard() {
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
    <section className="v16-codex-role-guide" aria-labelledby="v16-codex-role-guide-title">
      <header className="v16-codex-role-guide-header">
        <span>
          <strong id="v16-codex-role-guide-title">Agent 角色使用说明</strong>
          <small>你需要按角色开启会话，每个会话只做一种工作。</small>
        </span>
      </header>
      <div className="v16-codex-role-guide-body">
        <div className="v16-codex-role-grid">
          {codexRoleGuides.map((guide) => (
            <article className="v16-codex-role-card" key={guide.role}>
              <span>{guide.englishName}</span>
              <strong>{guide.title}</strong>
              <p>{guide.summary}</p>
              <pre className="v16-codex-role-instruction">{guide.startupInstruction}</pre>
              <ActionButton onClick={() => copyStartupInstruction(guide)} variant="secondary">
                复制 {guide.englishName} 启动指令
              </ActionButton>
            </article>
          ))}
        </div>
        {copyFeedback ? <p className="v16-feedback">{copyFeedback}</p> : null}
      </div>
    </section>
  );
}

function TasksPage({
  actionFeedback,
  actions,
  agentLocale,
  copyState,
  detailFocus,
  executeStatusState,
  mcpSessionsState,
  onDetailFocusHandled,
  onSelectProjectGroup,
  onTaskAction,
  onSelectTask,
  outputBundle,
  selectedProjectGroup,
  selectedProjectProjection,
  selectedTask,
  selectedTaskProjection,
  projectRoot,
  suggestions,
  taskTree,
  tasks,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  agentLocale: string;
  copyState: ButtonInteractionState;
  detailFocus: "delivery" | null;
  executeStatusState: ExecuteStatusState;
  mcpSessionsState: McpSessionsState;
  onDetailFocusHandled: () => void;
  onSelectProjectGroup: (projectId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  onSelectTask: (taskId: string) => void;
  outputBundle: OutputBundleState;
  projectRoot: string | null;
  selectedProjectGroup: TaskProjectGroup | null;
  selectedProjectProjection: ProjectProjection | null;
  selectedTask: V1Issue | null;
  selectedTaskProjection: TaskProjection | null;
  suggestions: ProjectInitializationContext[];
  taskTree: TaskProjectTreeViewModel | null;
  tasks: V1Issue[];
}) {
  return (
    <section className="v16-page v16-tasks-page" data-agentflow-page="tasks">
      <TaskList
        actionFeedback={actionFeedback}
        actions={actions}
        agentLocale={agentLocale}
        copyState={copyState}
        detailFocus={detailFocus}
        executeStatusState={executeStatusState}
        mcpSessionsState={mcpSessionsState}
        onDetailFocusHandled={onDetailFocusHandled}
        onSelectProjectGroup={onSelectProjectGroup}
        onSelectTask={onSelectTask}
        onTaskAction={onTaskAction}
        outputBundle={outputBundle}
        projectRoot={projectRoot}
        selectedProjectGroup={selectedProjectGroup}
        selectedProjectProjection={selectedProjectProjection}
        selectedTask={selectedTask}
        selectedTaskProjection={selectedTaskProjection}
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
  agentLocale,
  copyState,
  detailFocus,
  executeStatusState,
  mcpSessionsState,
  onDetailFocusHandled,
  onSelectProjectGroup,
  onSelectTask,
  onTaskAction,
  outputBundle,
  projectRoot,
  selectedProjectGroup,
  selectedProjectProjection,
  selectedTask,
  selectedTaskProjection,
  suggestions,
  taskTree,
  tasks,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  agentLocale: string;
  copyState: ButtonInteractionState;
  detailFocus: "delivery" | null;
  executeStatusState: ExecuteStatusState;
  mcpSessionsState: McpSessionsState;
  onDetailFocusHandled: () => void;
  onSelectProjectGroup: (projectId: string) => void;
  onSelectTask: (taskId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  outputBundle: OutputBundleState;
  projectRoot: string | null;
  selectedProjectGroup: TaskProjectGroup | null;
  selectedProjectProjection: ProjectProjection | null;
  selectedTask: V1Issue | null;
  selectedTaskProjection: TaskProjection | null;
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
                  {(() => {
                    const summary = taskMenuStatusSummary(task);
                    return (
                      <span className="v16-task-queue-main">
                        <span className="v16-task-queue-id-line">
                          <span className={`v16-task-queue-phase-dot ${taskTimelineToneForIssue(task)}`} aria-hidden="true" />
                          <span className="v16-list-item-id">{task.id}</span>
                        </span>
                        <span className="v16-task-queue-title-line">
                          <span>{task.title}</span>
                        </span>
                        {summary ? <small className="v16-task-queue-summary">{summary}</small> : null}
                      </span>
                    );
                  })()}
                  <span className="v16-task-queue-state">
                    <PriorityBadge priority={task.priority} />
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
        agentLocale={agentLocale}
        copyState={copyState}
        detailFocus={detailFocus}
        executeStatusState={executeStatusState}
        mcpSessionsState={mcpSessionsState}
        onDetailFocusHandled={onDetailFocusHandled}
        onTaskAction={onTaskAction}
        onSelectTask={onSelectTask}
        outputBundle={outputBundle}
        selectedProjectGroup={selectedProjectGroup}
        selectedProjectProjection={selectedProjectProjection}
        suggestions={showContextSuggestions ? suggestions : []}
        task={selectedTask}
        taskProjection={selectedTaskProjection}
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
  const workflowStatus = projectMenuWorkflowStatus(group);
  const timelineSections = taskProjectTimelineSections(group.issues);
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
        title={`${group.title} ${displayStatusLabelZh(workflowStatus)} ${progress}`}
        type="button"
      >
        {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        <StatusDot status={statusChipForDisplayStatus(workflowStatus)} />
        <span className="v16-task-project-main">
          <span className="v16-task-project-title">{group.title}</span>
        </span>
        <span className="v16-task-project-meta">
          <span>{progress}</span>
        </span>
      </button>
      {expanded ? (
        <div className="v16-task-project-children">
          {timelineSections.map((section) => (
            <TaskProjectTimelineSection
              key={section.id}
              onSelectTask={onSelectTask}
              section={section}
              selectedTaskId={selectedTaskId}
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

function TaskProjectTimelineSection({
  onSelectTask,
  section,
  selectedTaskId,
}: {
  onSelectTask: (taskId: string) => void;
  section: { id: "current" | "past" | "future"; issues: TaskIssueNode[] };
  selectedTaskId: string | null;
}) {
  return (
    <section className={`v16-task-timeline-section v16-task-timeline-section-${section.id}`} aria-label={section.id}>
      <div className="v16-task-timeline-items">
        {section.issues.map((issue) => (
          <TaskIssueNodeRow
            issue={issue}
            key={issue.id}
            onSelectTask={onSelectTask}
            selected={issue.id === selectedTaskId}
            timelineTone={taskTimelineToneForIssue(issue)}
          />
        ))}
      </div>
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
    <section className="v16-task-project-group v16-task-ungrouped-section" aria-label="单项任务">
      <div className="v16-task-project-row v16-task-ungrouped-row">
        <span className="v16-task-project-main">
          <span className="v16-task-project-title">单项任务</span>
        </span>
        <span className="v16-task-project-meta">{issues.length} 项</span>
      </div>
      <div className="v16-task-project-children v16-task-ungrouped-children">
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
  timelineTone,
}: {
  issue: TaskIssueNode;
  onSelectTask: (taskId: string) => void;
  selected: boolean;
  timelineTone?: TaskTimelineTone;
}) {
  const tone = timelineTone ?? taskTimelineToneForIssue(issue);
  const summary = taskMenuStatusSummary(issue);

  return (
    <button
      className={selected ? "v16-task-queue-row v16-task-node-row active" : "v16-task-queue-row v16-task-node-row"}
      onClick={() => onSelectTask(issue.id)}
      title={`${issue.id} ${issue.title} ${issueCategoryLabelZh(issue.issueCategory)} ${agentRoleLabelZh(issue.requiredAgentRole)}`}
      type="button"
    >
      <span className="v16-task-queue-main">
        <span className="v16-task-queue-id-line">
          <span className={`v16-task-queue-phase-dot ${tone}`} aria-hidden="true" />
          <span className="v16-list-item-id">{issue.id}</span>
        </span>
        <span className="v16-task-queue-title-line">
          <span>{issue.title}</span>
        </span>
        {summary ? <small className="v16-task-queue-summary">{summary}</small> : null}
      </span>
      <span className="v16-task-queue-state">
        <PriorityBadge priority={issue.priority} />
      </span>
    </button>
  );
}

function TaskDetail({
  actionFeedback,
  actions,
  agentLocale,
  copyState,
  detailFocus,
  executeStatusState,
  mcpSessionsState,
  onDetailFocusHandled,
  onSelectTask,
  onTaskAction,
  outputBundle,
  selectedProjectGroup,
  selectedProjectProjection,
  suggestions,
  task,
  taskProjection,
  taskTreeSelection,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  agentLocale: string;
  copyState: ButtonInteractionState;
  detailFocus: "delivery" | null;
  executeStatusState: ExecuteStatusState;
  mcpSessionsState: McpSessionsState;
  onDetailFocusHandled: () => void;
  onSelectTask: (taskId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  outputBundle: OutputBundleState;
  selectedProjectGroup: TaskProjectGroup | null;
  selectedProjectProjection: ProjectProjection | null;
  suggestions: ProjectInitializationContext[];
  task: V1Issue | null;
  taskProjection: TaskProjection | null;
  taskTreeSelection: TaskProjectTreeViewModel["selection"] | null;
}) {
  if (selectedProjectGroup) {
    return (
      <ProjectSummaryReader
        group={selectedProjectGroup}
        onSelectTask={onSelectTask}
        projection={selectedProjectProjection}
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
            <p>这些只是项目上下文，还不是已确认任务。</p>
          </header>
          <div className="v16-detail-document">
            <SectionList
              title="可整理的方向"
              items={suggestions.slice(0, 5).map((suggestion) => `${suggestion.title}：${suggestion.summary}`)}
            />
            <SectionList
              title="下一步"
              items={["先把其中一个方向整理成 SPEC，再生成任务。确认后才能进入执行。"]}
            />
          </div>
        </aside>
      );
    }
    return (
      <section className="v16-detail-pane v16-empty-detail-pane" aria-label="任务详情空态">
        <header>
          <h2>还没有任务</h2>
          <StatusBadge status="idle">等待需求</StatusBadge>
        </header>
        <div className="v16-detail-document">
          <SectionList title="下一步" items={["先确认需求，再生成任务。"]} />
        </div>
      </section>
    );
  }

  return (
    <TaskDetailReader
      actionFeedback={actionFeedback}
      actions={actions}
      agentLocale={agentLocale}
      copyState={copyState}
      detailFocus={detailFocus}
      executeStatusState={executeStatusState}
      mcpSessionsState={mcpSessionsState}
      onDetailFocusHandled={onDetailFocusHandled}
      onTaskAction={onTaskAction}
      outputBundle={outputBundle}
      task={task}
      taskProjection={taskProjection}
    />
  );
}

function ProjectSummaryReader({
  group,
  onSelectTask,
  projection,
  treeSelection,
}: {
  group: TaskProjectGroup;
  onSelectTask: (taskId: string) => void;
  projection: ProjectProjection | null;
  treeSelection: TaskProjectTreeViewModel["selection"] | null;
}) {
  const priority = groupHighestPriority(group.issues);
  const projectStatus = normalizeProjectDisplayStatus(projection?.status ?? projectDisplayStatusForGroup(group));
  const currentIssue = projection?.currentIssueId
    ? group.issues.find((issue) => issue.id === projection.currentIssueId) ?? null
    : projectRecommendedIssue(group, treeSelection);
  const nextIssue = projectNextScheduledIssue(
    group,
    currentIssue?.id ?? null,
    projection?.lanes.future ?? null,
  );
  const currentLaneCount = projection?.lanes.current.length
    ?? group.issues.filter((issue) => ["todo", "in_progress", "in_review", "blocked"].includes(issue.displayStatus)).length;
  const futureLaneCount = projection?.lanes.future.length
    ?? group.issues.filter((issue) => issue.displayStatus === "backlog").length;
  const completedLaneCount = projection
    ? projection.lanes.past.filter((issueId) => {
        const issue = group.issues.find((entry) => entry.id === issueId);
        return issue?.displayStatus === "done";
      }).length
    : group.issues.filter((issue) => issue.displayStatus === "done").length;
  const canceledLaneCount = group.issues.filter((issue) => issue.displayStatus === "cancel").length;
  const blockedLaneCount = projection?.lanes.blocked.length
    ?? group.issues.filter((issue) => issue.displayStatus === "blocked").length;
  const reviewIssueCount = projection
    ? projection.lanes.current.filter((issueId) => {
        const issue = group.issues.find((entry) => entry.id === issueId);
        return issue?.displayStatus === "in_review";
      }).length
    : group.issues.filter((issue) => issue.displayStatus === "in_review").length;
  const nextAction = projection?.nextAction ?? null;
  const completionHint = projection?.completionHint ?? null;
  const completion = projection?.completion ?? null;
  const completionLabel = projectCompletionStateLabelZh(
    completion?.currentState,
    completion?.latestOutcome,
  );
  const projectBrain = projection?.projectBrain ?? null;
  const brainActionLabel = projectBrainActionLabelZh(
    projectBrain?.nextRecommendedAction,
    projectBrain?.nextRecommendedActionLabel,
  );
  const brainActionReason = projectBrain?.nextRecommendedActionReason ?? "当前还没有 Project Brain 下一步说明。";
  const projectLoopReason =
    currentIssue || nextIssue
      ? "Project Brain 已确认上游目标和计划，项目已经进入任务循环。当前下一步由项目调度决定。"
      : brainActionReason;
  const brainOpenQuestions =
    projectBrain?.openQuestions.length ? projectBrain.openQuestions : ["当前没有待补充的开放问题。"];
  const blockerItems = projection?.blockers.length
    ? projection.blockers.map((blocker) => `${blocker.issueId}：${blocker.reason}`)
    : null;

  return (
    <aside className="v16-detail-pane" aria-label="项目调度视图">
      <header>
        <h2>{group.title}</h2>
        <p>{group.summary || group.objective || "查看这个项目当前推进到哪一步，以及下一条应该接哪项任务。"}</p>
        <div className="v16-detail-meta-strip">
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">状态</span>
            <StatusBadge status={statusChipForProjectStatus(projectStatus)}>{projectDisplayStatusLabelZh(projectStatus)}</StatusBadge>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">Project Brain</span>
            <strong className="v16-role-text">{projectBrainStatusLabelZh(projectBrain?.brainStatus)}</strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">任务进度</span>
            <strong className="v16-role-text">{group.counts.doneIssueCount}/{group.counts.issueCount || 0}</strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">最高优先级</span>
            <strong className={`v16-priority-text ${priorityTextClass(priority)}`}>{displayPriority(priority)}</strong>
          </span>
        </div>
      </header>
      <div className="v16-detail-document">
        <div className="v16-summary-grid">
          <MetricCard detail={`${group.counts.issueCount} 条任务`} label="项目状态" value={projectDisplayStatusLabelZh(projectStatus)} />
          <MetricCard detail={projectLoopReason} label="Brain 状态" value={projectBrainStatusLabelZh(projectBrain?.brainStatus)} />
          <MetricCard detail={`${reviewIssueCount} 条正在评审`} label="当前队列" value={currentLaneCount} />
          <MetricCard
            detail={completion?.nextRecommendedActionReason ?? completionHint ?? "当前还没有进入完成判断。"}
            label="完成判断"
            value={completionLabel}
          />
        </div>
        <section className="v16-task-stage-panel" aria-label="Project Brain 概览">
          <div className="v16-task-stage-panel-header">
            <span>Project Brain</span>
            <strong>{projectBrainStatusLabelZh(projectBrain?.brainStatus)}</strong>
          </div>
          <div className="v16-task-stage-grid">
            <SectionList
              title="当前目标"
              items={[
                group.objective || group.summary || group.title,
                `Goal：${projectBrainDocumentStatusLabelZh(projectBrain?.goalStatus)}`,
                `Project Brain：${projectBrainStatusLabelZh(projectBrain?.brainStatus)}`,
              ]}
            />
            <SectionList
              title="当前计划"
              items={[
                `Plan：${projectBrainDocumentStatusLabelZh(projectBrain?.planStatus)}`,
                `Decisions：${projectBrainDocumentStatusLabelZh(projectBrain?.decisionStatus)}`,
                `Project Health：${projectBrainDocumentStatusLabelZh(projectBrain?.healthStatus)}`,
              ]}
            />
            <SectionList
              title="下一步原因"
              items={[
                `Project Brain 建议：${brainActionLabel}`,
                `项目当前动作：${nextAction ?? "还没有进入任务循环。"}`,
                projectLoopReason,
                ...(projectBrain?.missingDocuments.length
                  ? [`缺失文档：${projectBrain.missingDocuments.join("、")}`]
                  : []),
              ]}
            />
          </div>
        </section>
        <section className="v16-task-stage-panel" aria-label="项目调度概览">
          <div className="v16-task-stage-panel-header">
            <span>项目调度</span>
            <strong>{currentIssue ? currentIssue.id : "等待首条任务"}</strong>
          </div>
          <div className="v16-task-stage-grid">
            <SectionList
              title="当前任务"
              items={
                currentIssue
                  ? [
                      `${currentIssue.id} · ${currentIssue.title}`,
                      `状态：${displayStatusLabelZh(currentIssue.displayStatus)}`,
                      `角色：${agentRoleLabelZh(currentIssue.requiredAgentRole)}`,
                    ]
                  : [nextAction ?? "当前没有正在推进的任务。"]
              }
            />
            <SectionList
              title="下一条任务"
              items={
                nextIssue
                  ? [
                      `${nextIssue.id} · ${nextIssue.title}`,
                      nextIssue.blockedBy.length ? `前置依赖：${nextIssue.blockedBy.join("、")}` : "当前没有前置依赖。",
                      `优先级：${displayPriority(nextIssue.priority)}`,
                    ]
                  : [`下一步：${brainActionLabel}`, projectLoopReason]
              }
            />
            <SectionList
              title="完成判断"
              items={[
                `状态：${completionLabel}`,
                completion?.nextRecommendedActionLabel
                  ? `下一步：${completion.nextRecommendedActionLabel}`
                  : `下一步：${nextAction ?? "等待项目继续推进。"}`,
                completion?.nextRecommendedActionReason
                  ?? completionHint
                  ?? "当前还没有完成判断记录。",
                ...(completion?.rationale.length
                  ? completion.rationale
                  : [
                      `当前队列：${currentLaneCount} 条`,
                      `未来队列：${futureLaneCount} 条`,
                      blockedLaneCount ? `阻断：${blockedLaneCount} 条` : "当前没有阻断任务。",
                      canceledLaneCount ? `取消：${canceledLaneCount} 条` : "当前没有取消任务。",
                    ]),
              ]}
            />
          </div>
        </section>
        <section className="v16-task-stage-panel" aria-label="项目状态流">
          <div className="v16-task-stage-panel-header">
            <span>状态流</span>
            <strong>{projectDisplayStatusLabelZh(projectStatus)}</strong>
          </div>
          <div className="v16-task-stage-grid">
            <SectionList
              title="当前"
              items={projectCurrentLaneItems(group, projection)}
            />
            <SectionList
              title="过去"
              items={projectPastLaneItems(group, projection)}
            />
            <SectionList
              title="未来"
              items={projectFutureLaneItems(group, projection)}
            />
          </div>
        </section>
        <details className="v16-task-package">
          <summary>高级详情</summary>
          <div className="v16-task-advanced-grid">
            <SectionList title="目标" items={[group.objective || group.summary || group.title]} />
            <SectionList title="范围" items={group.project.scope} />
            <SectionList title="非目标" items={group.project.nonGoals} />
            <SectionList title="依赖摘要" items={projectDependencySummaryItems(group)} />
            <SectionList title="优先级摘要" items={projectPrioritySummaryItems(group)} />
            <SectionList title="任务进度" items={projectProgressItems(group)} />
            <SectionList title="Project Brain 开放问题" items={brainOpenQuestions} />
            {blockerItems?.length ? <SectionList title="阻断摘要" items={blockerItems} /> : null}
            <SectionList
              title="项目信息"
              items={[
                `项目 ID：${group.id}`,
                `来源 SPEC：${group.sourceSpecId ?? "未记录"}`,
                `任务数量：${group.counts.issueCount}`,
              ]}
            />
            {projectBrain ? (
              <SectionList
                title="Project Brain 文档"
                items={[
                  `GOAL.md：${projectBrain.goalPath}`,
                  `PLAN.md：${projectBrain.planPath}`,
                  `DECISIONS.md：${projectBrain.decisionsPath}`,
                  `PROJECT_HEALTH.md：${projectBrain.healthPath}`,
                ]}
              />
            ) : null}
            {group.warnings.length || group.missingIssueIds.length ? (
              <SectionList title="缺失引用" items={projectWarningItems(group)} />
            ) : null}
          </div>
        </details>
      </div>
      <ActionBar sticky>
        <ActionButton
          disabled={!currentIssue}
          onClick={() => {
            if (currentIssue) {
              onSelectTask(currentIssue.id);
            }
          }}
          variant="primary"
        >
          查看当前任务
        </ActionButton>
      </ActionBar>
    </aside>
  );
}

function TaskDetailReader({
  actionFeedback,
  actions,
  agentLocale,
  copyState,
  detailFocus,
  executeStatusState,
  mcpSessionsState,
  onDetailFocusHandled,
  onTaskAction,
  outputBundle,
  task,
  taskProjection,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  agentLocale: string;
  copyState: ButtonInteractionState;
  detailFocus: "delivery" | null;
  executeStatusState: ExecuteStatusState;
  mcpSessionsState: McpSessionsState;
  onDetailFocusHandled: () => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  outputBundle: OutputBundleState;
  task: V1Issue;
  taskProjection: TaskProjection | null;
}) {
  const [handoffOpen, setHandoffOpen] = useState(false);
  const deliveryFocusRef = useRef<HTMLDivElement | null>(null);
  const effectiveDisplayStatus = taskProjection?.currentState ?? task.displayStatus ?? "backlog";
  const effectiveTask: V1Issue = taskProjection
    ? {
        ...task,
        displayStatus: effectiveDisplayStatus,
        latestRunId: taskProjection.latestRunId ?? task.latestRunId,
      }
    : task;
  const handoffError = useMemo(() => taskHandoffValidationError(task), [task]);
  const statusContract = useMemo(() => buildTaskStatusContract(effectiveTask), [effectiveTask]);
  const session = useMemo(
    () => pickLatestMcpSessionForIssue(mcpSessionsState.sessions, task.id),
    [mcpSessionsState.sessions, task.id],
  );
  const detailDescriptionItems = useMemo(
    () => [
      ...taskAuditDescriptionItems(effectiveTask),
      ...(effectiveTask.auditTrigger ? [["触发来源", auditTriggerLabel(effectiveTask.auditTrigger)] as [string, string]] : []),
    ],
    [effectiveTask],
  );
  const delivery = null;
  const evidence = null;
  const audit = null;
  const stageOutputItems = useMemo(
    () => taskCurrentStageOutputItems(effectiveTask, session, delivery, evidence, audit, taskProjection),
    [audit, delivery, effectiveTask, evidence, session, taskProjection],
  );
  const executionProjection = useMemo(
    () =>
      buildTaskExecutionProjection({
        executeStatusError: executeStatusState.error,
        executeStatusSource: executeStatusState.source,
        executeWorkspaceStatus: executeStatusState.status?.status ?? null,
        executeWorkspaceWarnings: executeStatusState.status?.warnings ?? [],
        mcpSessionsError: mcpSessionsState.error,
        mcpSessionsSource: mcpSessionsState.source,
        projection: taskProjection,
        session,
        task: effectiveTask,
      }),
    [effectiveTask, executeStatusState, mcpSessionsState, session],
  );
  const deliveryProjection = useMemo(
    () => buildTaskDeliveryProjection({ audit, delivery, evidence, projection: taskProjection, session, task: effectiveTask }),
    [audit, delivery, effectiveTask, evidence, session, taskProjection],
  );
  const reviewItems = useMemo(
    () => deliveryReviewItems(effectiveTask, null, session, taskProjection),
    [effectiveTask, session, taskProjection],
  );
  const finalDeliveryArtifact = useMemo(
    () => outputBundle.deliveryArtifacts[deliveryProjection.deliveryRunId ?? effectiveTask.latestRunId ?? ""]
      ?? null,
    [deliveryProjection.deliveryRunId, effectiveTask.latestRunId, outputBundle.deliveryArtifacts],
  );
  const workflowYaml = useMemo(
    () =>
      buildTaskWorkflowYamlModel({
        contract: statusContract,
        deliveryProjection,
        executionProjection,
        task: effectiveTask,
      }),
    [deliveryProjection, effectiveTask, executionProjection, statusContract],
  );
  const handoffContent = useMemo(() => {
    if (!handoffOpen) {
      return "";
    }
    return handoffError ?? buildCodexHandoff(task, agentLocale);
  }, [agentLocale, handoffError, handoffOpen, task]);

  useEffect(() => {
    if (detailFocus !== "delivery") {
      return;
    }
    const frame = window.requestAnimationFrame(() => {
      deliveryFocusRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
      onDetailFocusHandled();
    });
    return () => window.cancelAnimationFrame(frame);
  }, [detailFocus, onDetailFocusHandled, task.id]);

  return (
    <aside className="v16-detail-pane" aria-label="任务工作流">
      <header>
        <h2>{task.title}</h2>
        <p>{task.id} · {task.goal || task.title}</p>
        <div className="v16-detail-meta-strip">
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">类型：</span>
            <strong className="v16-role-text">{issueCategoryLabelZh(task.issueCategory)}</strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">状态</span>
            <StatusBadge status={statusChipForDisplayStatus(effectiveDisplayStatus)}>
              {displayStatusLabelZh(effectiveDisplayStatus)}
            </StatusBadge>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">优先级</span>
            <strong className={`v16-priority-text ${priorityTextClass(task.priority)}`}>
              {displayPriority(task.priority)}
            </strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">角色</span>
            <strong className="v16-role-text">{agentRoleLabelZh(task.requiredAgentRole)}</strong>
          </span>
          <span className="v16-detail-meta-item">
            <span className="v16-detail-meta-label">更新时间</span>
            <strong className="v16-role-text">
              {task.updatedAt || task.createdAt ? formatTimestamp(task.updatedAt ?? task.createdAt ?? 0) : "未记录"}
            </strong>
          </span>
        </div>
      </header>
      <div className="v16-detail-document">
        <div className="v16-task-workspace-shell">
          <div className="v16-task-workspace-main">
            <IssueStatusFlow
              deliveryProjection={deliveryProjection}
              task={effectiveTask}
              executeItems={executionProjection.summaryItems}
              projection={taskProjection}
              reviewItems={reviewItems}
              stageItems={stageOutputItems}
              status={effectiveDisplayStatus}
            />
          </div>
          <aside className="v16-task-workspace-sidebar" aria-label="当前阶段摘要">
            <TaskFlowSidebar
              artifact={finalDeliveryArtifact}
              deliveryProjection={deliveryProjection}
              reviewItems={reviewItems}
              stageItems={stageOutputItems}
              status={effectiveDisplayStatus}
              task={effectiveTask}
              taskProjection={taskProjection}
            />
          </aside>
        </div>
        <div ref={deliveryFocusRef} className="v16-task-delivery-focus-region" />
        <details className="v16-task-package">
          <summary>高级详情</summary>
          <div className="v16-task-advanced-grid">
            <SectionList
              title="状态说明"
              items={[
                statusContract.businessMeaning,
                `责任角色：${statusContract.ownerRoleLabel}`,
                `下一步入口：${statusContract.nextEntry}`,
              ]}
            />
            <SectionList title="目标" items={[task.goal || task.title]} />
            <SectionList title="范围" items={task.scope} />
            <SectionList title="非目标" items={task.nonGoals} />
            <SectionList title="验收标准" items={task.acceptanceCriteria} />
            <SectionList title="验证命令" items={task.validationCommands} />
            <SectionList title="预期产物路径" items={taskOutputItems(task)} />
            <SectionList title="交付路径明细" items={deliveryProjection.packageItems} />
            <SectionList title="交付状态摘要" items={deliveryProjection.summaryItems} />
            <SectionList title="证据要求" items={task.evidenceRequired} />
            {detailDescriptionItems.length ? <SectionList title="补充信息" items={detailDescriptionItems.map(([label, value]) => `${label}：${value}`)} /> : null}
            {task.issueCategory === "spec" ? <SectionList title="执行流程" items={taskExecutionPipelineItems(task)} /> : null}
            <TaskWorkflowPanelShell yamlModel={workflowYaml} />
            <section className="v16-task-package v16-task-package-nested">
              <h3>Agent 任务包</h3>
              <ActionButton
                onClick={() => setHandoffOpen((current) => !current)}
                variant="secondary"
              >
                {handoffOpen ? "收起任务包" : "展开任务包"}
              </ActionButton>
              {handoffOpen ? (
                <CopyableCodeBlock
                  content={handoffContent}
                  maxHeight={210}
                  title="Agent 任务包"
                />
              ) : null}
            </section>
          </div>
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

function TaskWorkflowPanelShell({
  yamlModel,
}: {
  yamlModel: TaskWorkflowYamlModel;
}) {
  return (
    <section className="v16-task-workflow-panel-shell" aria-label="YAML workflow 面板">
      <div className="v16-task-workflow-panel-shell-header">
        <span>YAML workflow</span>
        <strong>{yamlModel.summary}</strong>
      </div>
      <CopyableCodeBlock content={yamlModel.content} language="yaml" maxHeight={420} title={yamlModel.fileName} />
    </section>
  );
}

function TaskFlowSidebar({
  artifact,
  deliveryProjection,
  reviewItems,
  stageItems,
  status,
  task,
  taskProjection,
}: {
  artifact: DeliveryArtifactState | null;
  deliveryProjection: TaskDeliveryProjection;
  reviewItems: string[];
  stageItems: string[];
  status: IssueDisplayStatus;
  task: V1Issue;
  taskProjection: TaskProjection | null;
}) {
  const isDone = task.displayStatus === "done";
  const statusLabel = isDone
    ? deliveryProjection.missingItems.length
      ? "交付待补齐"
      : "最终交付已就绪"
    : task.displayStatus === "in_review"
      ? "交付已生成，等待评审收口"
      : "等待进入交付阶段";
  const releaseTitle = artifact?.releaseNote?.title ?? null;
  const releaseSummary = artifact?.releaseNote?.summaryLines.length
    ? artifact.releaseNote.summaryLines.slice(0, 3)
    : deliveryProjection.deliveryRunId
      ? ["交付包已登记，release note 摘要未读取到。"]
      : ["任务完成后显示交付包摘要。"];
  const packageSummary = [
    releaseTitle ? `交付标题：${releaseTitle}` : "交付标题：等待生成",
    deliveryProjection.deliveryRunId ? `交付编号：${deliveryDisplayId(deliveryProjection.deliveryRunId)}` : "交付编号：未生成",
    `验证证据：${deliveryProjection.evidencePath ? "已记录" : "未记录"}`,
    `评审链接：${deliveryProjection.prUrl ? "已记录" : "未记录"}`,
    `合并状态：${deliveryProjection.mergeState ? artifactStatusLabel(deliveryProjection.mergeState) : "未记录"}`,
  ];
  const waitingItems =
    task.displayStatus === "in_review" || task.displayStatus === "done"
      ? []
      : ["当前还没有进入最终交付阶段。", "任务完成本地验证并进入评审后，这里会显示交付摘要。"];
  const auditItems = taskAuditSummaryItems(task, taskProjection);
  const nextStepItems = [`当前状态：${displayStatusLabelZh(status)}`, ...stageItems];

  return (
    <div className="v16-task-side-rail">
      <section className="v16-task-stage-panel" aria-label="当前阶段摘要">
        <div className="v16-task-stage-panel-header">
          <span>当前阶段</span>
          <strong>{displayStatusLabelZh(status)}</strong>
        </div>
        <div className="v16-task-stage-grid v16-task-stage-grid-compact">
          <SectionList title="当前动作" items={nextStepItems} />
          <SectionList title="评审状态" items={reviewItems} />
        </div>
      </section>
      <section className={isDone ? "v16-final-delivery-card done" : "v16-final-delivery-card"} aria-label="交付摘要">
        <div>
          <span>交付</span>
          <strong>{deliveryProjection.deliveryRunId ?? "未生成"}</strong>
          <small>{statusLabel}</small>
        </div>
        <ul>
          {releaseSummary.map((item) => (
            <li key={item}>{item}</li>
          ))}
          {packageSummary.map((item) => (
            <li key={item}>{item}</li>
          ))}
          {waitingItems.map((item) => (
            <li key={item}>{item}</li>
          ))}
          <li>{deliveryProjection.missingItems.length ? `待补齐：${deliveryProjection.missingItems.length} 项` : "交付摘要已和当前状态对齐。"}</li>
        </ul>
      </section>
      <SectionList title="审计摘要" items={auditItems} />
    </div>
  );
}

function IssueStatusFlow({
  deliveryProjection,
  task,
  executeItems,
  projection,
  reviewItems,
  stageItems,
  status,
}: {
  deliveryProjection: TaskDeliveryProjection;
  task: V1Issue;
  executeItems: string[];
  projection: TaskProjection | null;
  reviewItems: string[];
  stageItems: string[];
  status: IssueDisplayStatus;
}) {
  const contract = buildTaskStatusContract(task);
  const projectedStatus = projection?.currentState ?? status;
  const steps = projection
    ? buildProjectionTaskStatusTimeline(projection, contract)
    : buildTaskStatusTimeline(projectedStatus, contract);
  const [selectedStepId, setSelectedStepId] = useState<IssueDisplayStatus>(projectedStatus);

  useEffect(() => {
    setSelectedStepId(projectedStatus);
  }, [projectedStatus, task.id]);

  const selectedStepLabel = steps.find((step) => step.id === selectedStepId)?.label ?? contract.label;

  return (
    <section className="v16-issue-status-flow" aria-label="任务状态流转">
      <div className="v16-issue-status-flow-header">
        <span>状态流 / 事件流</span>
        <strong>{selectedStepLabel}</strong>
      </div>
      <ol>
        {steps.map((step) => {
          const state = step.state === "exception" ? step.id : step.state;
          const expanded = step.id === selectedStepId;
          const projectionItem = projection?.timeline.find((item) => item.state === step.id) ?? null;
          const detail = expanded
            ? enhanceTaskStatusStepDetailWithProjection({
                detail: buildTaskStatusStepDetail({
                  currentStatus: projectedStatus,
                  deliveryProjection,
                  executeItems,
                  reviewItems,
                  stageItems,
                  task,
                  viewedStatus: step.id,
                }),
                projection,
                projectionItem,
              })
            : null;
          return (
            <li
              aria-current={expanded ? "step" : undefined}
              className={`v16-issue-status-step ${state}${expanded ? " selected" : ""}`}
              key={step.id}
            >
              <span className="v16-issue-status-dot" aria-hidden="true" />
              <div className="v16-issue-status-step-body">
                <button
                  aria-expanded={expanded}
                  className="v16-issue-status-step-toggle"
                  onClick={() => setSelectedStepId(step.id)}
                  type="button"
                >
                  <strong>{step.label}</strong>
                  <small>{step.note}</small>
                </button>
                {expanded && detail ? (
                  <div className="v16-issue-status-step-details">
                    <div className="v16-issue-status-event-stream">
                      <ul className="v16-issue-status-meta-list">
                        {detail.descriptionItems.map(([label, value]) => (
                          <li className="v16-issue-status-meta-item" key={`${label}-${value}`}>
                            <span className="v16-issue-status-event-label">{label}</span>
                            <span className="v16-issue-status-event-text">{value}</span>
                          </li>
                        ))}
                      </ul>
                      <div className="v16-issue-status-event-groups">
                        {detail.sections.map((section) => (
                          <section className="v16-issue-status-event-group" key={section.title}>
                            <h3>{section.title}</h3>
                            <ol className="v16-issue-status-event-list">
                              {section.items.map((item) => (
                                <li className="v16-issue-status-event-item" key={item}>
                                  <span className="v16-issue-status-event-text">{item}</span>
                                </li>
                              ))}
                            </ol>
                          </section>
                        ))}
                      </div>
                    </div>
                  </div>
                ) : null}
              </div>
            </li>
          );
        })}
      </ol>
    </section>
  );
}

function buildProjectionTaskStatusTimeline(
  projection: TaskProjection,
  contract: ReturnType<typeof buildTaskStatusContract>,
) {
  const knownStates = new Set<IssueDisplayStatus>([
    "backlog",
    "todo",
    "in_progress",
    "in_review",
    "done",
    "blocked",
    "cancel",
  ]);
  const timeline = projection.timeline.filter((item) => knownStates.has(item.state));
  if (!timeline.length) {
    return buildTaskStatusTimeline(projection.currentState, contract);
  }

  return timeline.map((item) => ({
    id: item.state,
    label: displayStatusLabelZh(item.state),
    note: item.summary || projectionPhaseLabel(item.phase),
    state: projectionPhaseStepState(item.phase),
  }));
}

function projectionPhaseStepState(phase: ProjectionPhase) {
  switch (phase) {
    case "past":
      return "done" as const;
    case "current":
      return "current" as const;
    case "exception":
      return "exception" as const;
    case "future":
    default:
      return "idle" as const;
  }
}

function projectionPhaseLabel(phase: ProjectionPhase) {
  const labels: Record<ProjectionPhase, string> = {
    current: "当前阶段",
    exception: "异常阶段",
    future: "等待进入",
    past: "已完成阶段",
  };
  return labels[phase];
}

function taskTimelineEventLine(event: TaskTimelineItem["events"][number]) {
  const time = event.timestamp ? formatTimestamp(event.timestamp) : "未记录时间";
  const actor = event.actorRole || event.actorKind || "system";
  return `${time} · ${actor} · ${event.summary || event.eventType}`;
}

function enhanceTaskStatusStepDetailWithProjection({
  detail,
  projection,
  projectionItem,
}: {
  detail: ReturnType<typeof buildTaskStatusStepDetail>;
  projection: TaskProjection | null;
  projectionItem: TaskTimelineItem | null;
}) {
  if (!projection || !projectionItem) {
    return detail;
  }

  if (projectionItem.phase === "future") {
    return {
      descriptionItems: [
        ...detail.descriptionItems,
        ["事件阶段", projectionPhaseLabel(projectionItem.phase)] as [string, string],
        [
          "进入时间",
          projectionItem.enteredAt ? formatTimestamp(projectionItem.enteredAt) : "等待进入",
        ] as [string, string],
      ],
      sections: detail.sections,
    };
  }

  const eventItems = projectionItem.events.length
    ? projectionItem.events.map((event) => taskTimelineEventLine(event))
    : projectionItem.phase === "current"
      ? ["当前阶段已进入，等待新的事件写入。"]
      : ["这个阶段没有记录到历史事件。"];
  const artifactItems = [
    ...projectionItem.liveRefs,
    ...projectionItem.events.flatMap((event) => event.artifactRefs),
  ];
  const uniqueArtifactItems = Array.from(new Set(artifactItems));
  const artifactLines = uniqueArtifactItems.length
    ? uniqueArtifactItems.map((ref) => `产物：${ref}`)
    : projectionItem.phase === "current"
      ? ["当前阶段还没有产物写入。"]
      : ["这个阶段没有保留产物记录。"];
  const runtimeItems = projectionRuntimeSessionItems(projection, projectionItem);
  const publicDeliveryItems =
    projectionItem.state === "in_review" || projectionItem.state === "done"
      ? projectionPublicDeliveryItems(projection)
      : [];
  const auditItems =
    projectionItem.state === "done"
      ? projectionAuditItems(projection)
      : [];

  return {
    descriptionItems: [
      ...detail.descriptionItems,
      ["事件阶段", projectionPhaseLabel(projectionItem.phase)] as [string, string],
      [
        "进入时间",
        projectionItem.enteredAt ? formatTimestamp(projectionItem.enteredAt) : "未记录",
      ] as [string, string],
      ["事件来源", projection.currentTransition ?? "等待事件"] as [string, string],
    ],
    sections: [
      {
        title: "状态事件",
        items: [projectionItem.summary, ...eventItems],
      },
      {
        title: "事件产物",
        items: artifactLines,
      },
      ...(runtimeItems.length
        ? [
            {
              title: projectionItem.phase === "current" ? "当前执行态" : "阶段执行态",
              items: runtimeItems,
            },
          ]
        : []),
      ...(publicDeliveryItems.length
        ? [
            {
              title: "交付记录",
              items: publicDeliveryItems,
            },
          ]
        : []),
      ...(auditItems.length
        ? [
            {
              title: "审计摘要",
              items: auditItems,
            },
          ]
        : []),
      ...detail.sections,
    ],
  };
}

function projectionPublicDeliveryItems(projection: TaskProjection) {
  const delivery = projection.publicDelivery;
  return [
    delivery.prUrl ? `PR/MR：${delivery.prUrl}` : "PR/MR：未记录",
    delivery.mergeCommit ? `合并提交：${delivery.mergeCommit}` : "合并提交：未记录",
    delivery.evidencePath ? `验证证据：${delivery.evidencePath}` : "验证证据：未记录",
    delivery.changelogPath ? `CHANGELOG：${delivery.changelogPath}` : "CHANGELOG：未记录",
    delivery.releaseNotesUrl ? `Release notes：${delivery.releaseNotesUrl}` : "Release notes：未记录",
  ];
}

function projectionAuditItems(projection: TaskProjection) {
  const audit = projection.audit;
  if (!audit || audit.status === "not-requested") {
    return ["审计仍是独立流程，当前没有审计请求。"];
  }
  return [
    `审计状态：${artifactStatusLabel(audit.status)}`,
    audit.latestAuditId ? `审计编号：${audit.latestAuditId}` : "审计编号：未记录",
    audit.reportPath ? `审计报告：${audit.reportPath}` : "审计报告：未记录",
  ];
}

function projectionRuntimeSessionItems(
  projection: TaskProjection,
  projectionItem: TaskTimelineItem,
) {
  if (projectionItem.phase === "future") {
    return [];
  }

  const items = [
    projection.runtime.runId ? `Run：${projection.runtime.runId}` : null,
    projection.runtime.runStatus ? `执行状态：${executeProgressLabel(projection.runtime.runStatus)}` : null,
    projection.session.status ? `会话状态：${mcpSessionStatusLabelZh(projection.session.status)}` : null,
    projection.session.sessionId ? `会话 ID：${projection.session.sessionId}` : null,
    projection.session.branchName ? `工作分支：${projection.session.branchName}` : null,
    projection.session.planPath ? `执行计划：${projection.session.planPath}` : null,
    projection.session.logPath ? `事件日志：${projection.session.logPath}` : null,
  ].filter((item): item is string => Boolean(item));

  return Array.from(new Set(items));
}

function taskStatusTransitionLabel(status: IssueDisplayStatus, nextEntry: string) {
  const nextStatus: Partial<Record<IssueDisplayStatus, string>> = {
    backlog: "准备开工",
    blocked: "待恢复",
    cancel: "已结束",
    done: "已完成",
    in_progress: "正在评审",
    in_review: "已完成",
    todo: "正在做",
  };
  const label = nextStatus[status];
  return label ? `要进入${label}：${nextEntry}` : nextEntry;
}

function buildTaskStatusStepDetail({
  currentStatus,
  deliveryProjection,
  executeItems,
  reviewItems,
  stageItems,
  task,
  viewedStatus,
}: {
  currentStatus: IssueDisplayStatus;
  deliveryProjection: TaskDeliveryProjection;
  executeItems: string[];
  reviewItems: string[];
  stageItems: string[];
  task: V1Issue;
  viewedStatus: IssueDisplayStatus;
}) {
  const stepTask = viewedStatus === currentStatus ? task : { ...task, displayStatus: viewedStatus };
  const contract = buildTaskStatusContract(stepTask);
  if (isFutureTaskStatus(viewedStatus, currentStatus)) {
    return {
      descriptionItems: [
        ["当前状态", contract.businessMeaning],
        ["等待条件", `当前任务还处在${displayStatusLabelZh(currentStatus)}，还没有进入${contract.label}。`],
        ["进入下一步", taskStatusTransitionLabel(viewedStatus, contract.nextEntry)],
        ["信息流", "这个阶段还没开始，先看等待摘要。"],
      ] as Array<[string, string]>,
      sections: [
        {
          title: "等待摘要",
          items: taskStatusWaitingSummaryItems(viewedStatus, currentStatus, task),
        },
      ],
    };
  }

  const sectionItems = taskStatusSectionItems({
    contract,
    currentStatus,
    executeItems,
    reviewItems,
    stageItems,
    task,
    viewedStatus,
  });
  const live = viewedStatus === currentStatus && ["todo", "in_progress", "in_review", "blocked"].includes(currentStatus);

  return {
    descriptionItems: [
      ["当前状态", contract.businessMeaning],
      ["当前动作", contract.stageAction],
      ["进入下一步", taskStatusTransitionLabel(viewedStatus, contract.nextEntry)],
      ["信息流", live ? "当前阶段展示实时信息流。" : "这个阶段已结束，保留阶段日志。"],
    ] as Array<[string, string]>,
    sections: buildTaskStatusDetailSections({
      deliveryProjection,
      executeItems: sectionItems.executeItems,
      reviewItems: sectionItems.reviewItems,
      stageItems: sectionItems.stageItems,
      status: viewedStatus,
    }),
  };
}

function buildTaskStatusDetailSections({
  deliveryProjection,
  executeItems,
  reviewItems,
  stageItems,
  status,
}: {
  deliveryProjection: TaskDeliveryProjection;
  executeItems: string[];
  reviewItems: string[];
  stageItems: string[];
  status: IssueDisplayStatus;
}) {
  const sections: Array<{ title: string; items: string[] }> = [];

  if (status === "backlog" || status === "todo") {
    sections.push({
      title: "准备信息",
      items: executeItems,
    });
  }

  sections.push({
    title: "阶段摘要",
    items:
      status === "in_review" || status === "done"
        ? [...stageItems, ...deliveryProjection.summaryItems]
        : stageItems,
  });

  if (status === "in_review" || status === "done") {
    sections.push({
      title: "评审与写回",
      items: reviewItems,
    });
  }

  if (status === "blocked" || status === "cancel") {
    sections.push({
      title: "状态说明",
      items: executeItems,
    });
  }

  return sections;
}

function taskAuditSummaryItems(task: V1Issue, projection?: TaskProjection | null) {
  if (task.issueCategory === "audit") {
    return [
      `审计目标：${task.auditId ?? "未记录"}`,
      `输出目录：${task.auditOutputDir ?? "未记录"}`,
      `当前状态：${displayStatusLabelZh(task.displayStatus)}`,
    ];
  }

  const audit = projection?.audit ?? null;
  if (!audit || audit.status === "not-requested") {
    return ["审计仍是独立流程，当前没有审计请求。"];
  }

  return [
    `状态：${artifactStatusLabel(audit.status)}`,
    audit.latestAuditId ? `审计编号：${audit.latestAuditId}` : "审计编号：未记录",
    audit.reportPath ? `审计报告：${audit.reportPath}` : "审计报告：未记录",
  ];
}

function isFutureTaskStatus(viewedStatus: IssueDisplayStatus, currentStatus: IssueDisplayStatus) {
  if (currentStatus === "blocked" || currentStatus === "cancel") {
    return false;
  }
  const order = ["backlog", "todo", "in_progress", "in_review", "done"] as const;
  const currentIndex = order.indexOf(currentStatus as (typeof order)[number]);
  const viewedIndex = order.indexOf(viewedStatus as (typeof order)[number]);
  return currentIndex >= 0 && viewedIndex > currentIndex;
}

function taskStatusWaitingSummaryItems(viewedStatus: IssueDisplayStatus, currentStatus: IssueDisplayStatus, task: V1Issue) {
  const items = [
    `当前任务还处在${displayStatusLabelZh(currentStatus)}。`,
    `要进入${displayStatusLabelZh(viewedStatus)}，先完成当前阶段。`,
  ];

  if (task.dependencies.length) {
    items.push(`前置依赖：${task.dependencies.join("、")}`);
  }

  switch (viewedStatus) {
    case "todo":
      items.push("任务合同确认后，这里会展示前置检测和开工准备。");
      break;
    case "in_progress":
      items.push("正式开工后，这里会开始显示执行中的实时信息流。");
      break;
    case "in_review":
      items.push("本地验证完成并创建评审请求后，这里会保留评审日志。");
      break;
    case "done":
      items.push("合并和写回完成后，这里会保留最终结果和交付日志。");
      break;
    default:
      items.push("当前阶段还没有更多状态信息。");
      break;
  }

  return items;
}

function taskStatusSectionItems({
  contract,
  currentStatus,
  executeItems,
  reviewItems,
  stageItems,
  task,
  viewedStatus,
}: {
  contract: ReturnType<typeof buildTaskStatusContract>;
  currentStatus: IssueDisplayStatus;
  executeItems: string[];
  reviewItems: string[];
  stageItems: string[];
  task: V1Issue;
  viewedStatus: IssueDisplayStatus;
}) {
  const effectiveExecuteItems = executeItems.length ? executeItems : taskStatusExecuteLogItems(viewedStatus);
  const effectiveReviewItems = reviewItems.length ? reviewItems : taskStatusReviewLogItems(viewedStatus);
  const effectiveStageItems = stageItems.length ? stageItems : contract.stageOutputs;
  const runLabel = task.latestRunId ? `执行 Run：${task.latestRunId}` : "执行 Run：未记录。";

  switch (viewedStatus) {
    case "backlog":
      return {
        executeItems: [
          currentStatus === "backlog" ? "当前还没有进入执行。" : `这个阶段已经结束，后续已推进到${displayStatusLabelZh(currentStatus)}。`,
          currentStatus === "backlog" ? "等待整理任务边界。" : "任务边界已整理完成。",
        ],
        reviewItems: ["当前阶段不涉及评审。"],
        stageItems: [
          "任务合同已生成。",
          task.dependencies.length ? `前置依赖：${task.dependencies.join("、")}` : "前置依赖：无。",
          task.allowedFiles.length ? "允许变更范围已锁定。" : "允许变更范围待补充。",
        ],
      };
    case "todo":
      return {
        executeItems:
          currentStatus === "todo"
            ? effectiveExecuteItems
            : [runLabel, "前置检测已完成。", "任务已经进入正式执行阶段。"],
        reviewItems: ["当前阶段不涉及评审。"],
        stageItems: [
          task.contextPackPath ? `Context Pack：${task.contextPackPath}` : "Context Pack：等待生成。",
          task.latestRunId ? runLabel : "执行 Run：等待创建。",
          currentStatus === "todo" ? `执行准备：${executeProgressLabel(task.executeStatus)}` : "执行准备已完成。",
        ],
      };
    case "in_progress":
      return {
        executeItems: effectiveExecuteItems,
        reviewItems: effectiveReviewItems,
        stageItems:
          currentStatus === "in_progress"
            ? effectiveStageItems
            : [runLabel, "代码改动已完成。", "本地验证结果已归档。"],
      };
    case "in_review":
      return {
        executeItems: effectiveExecuteItems,
        reviewItems: effectiveReviewItems,
        stageItems:
          currentStatus === "in_review"
            ? effectiveStageItems
            : [runLabel, "交付材料已生成。", "等待合并和最终写回。"],
      };
    case "done":
      return {
        executeItems: effectiveExecuteItems,
        reviewItems: effectiveReviewItems,
        stageItems: effectiveStageItems,
      };
    case "blocked":
    case "cancel":
    default:
      return {
        executeItems: effectiveExecuteItems,
        reviewItems: effectiveReviewItems,
        stageItems: effectiveStageItems,
      };
  }
}

function taskStatusExecuteLogItems(status: IssueDisplayStatus) {
  switch (status) {
    case "backlog":
      return ["还没有执行记录。", "等待任务合同确认后进入准备开工。"];
    case "todo":
      return ["当前还没有正式开工。", "前置检测通过后会创建 run 并进入正在做。"];
    case "in_progress":
      return ["执行线程已拉起。", "正在处理改动和本地验证。"];
    case "in_review":
      return ["本地验证已完成。", "等待 PR/MR 合并或最终核对。"];
    case "done":
      return ["执行链路已结束。", "最终状态和交付已写回。"];
    case "blocked":
      return ["执行被关键因素阻断。", "解除阻断前不会继续推进。"];
    case "cancel":
      return ["任务已取消。", "不会再继续执行。"];
    default:
      return ["暂无执行记录。"];
  }
}

function taskStatusReviewLogItems(status: IssueDisplayStatus) {
  switch (status) {
    case "in_review":
      return ["评审请求已创建。", "等待合并后写回已完成。"];
    case "done":
      return ["评审已收口。", "合并凭证和写回都已完成。"];
    case "blocked":
      return ["当前不进入评审。", "先处理阻断原因。"];
    case "cancel":
      return ["当前不进入评审。", "任务已取消。"];
    default:
      return ["当前还没有评审记录。", "进入评审后这里会显示评审动作。"];
  }
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
  mcpSessionsState,
  onOpenAudit,
  onSelectDelivery,
  outputBundle,
  selectedDeliveryRunId,
  selectedTask,
  tasks,
}: {
  mcpSessionsState: McpSessionsState;
  onOpenAudit: () => void;
  onSelectDelivery: (runId: string) => void;
  outputBundle: OutputBundleState;
  selectedDeliveryRunId: string | null;
  selectedTask: V1Issue | null;
  tasks: V1Issue[];
}) {
  const deliveries = taskPublicDeliveryEntries(tasks);
  const evidence = sortOutputEntriesByLatest(outputBundle.outputIndex?.evidence ?? []);
  const deliveryInteractionState = buildDeliveryInteractionState(deliveries, selectedDeliveryRunId);
  const selectedDelivery = deliveryInteractionState.selectedDelivery ?? findDeliveryEntryForTask(deliveries, selectedTask);
  const deliveryTask = selectedDelivery
    ? tasks.find((task) => task.id === selectedDelivery.issueId) ?? null
    : selectedTask;
  const taskSession = deliveryTask ? pickLatestMcpSessionForIssue(mcpSessionsState.sessions, deliveryTask.id) : null;

  return (
    <section className="v16-page v16-split-page" data-agentflow-page="delivery">
      <DeliveryList
        deliveries={deliveries}
        onSelectDelivery={onSelectDelivery}
        selectedDeliveryRunId={selectedDelivery?.runId ?? deliveryInteractionState.selectedDeliveryRunId}
      />
      <DeliveryDetail
        audits={outputBundle.auditIndex?.audits ?? []}
        delivery={selectedDelivery}
        evidence={evidence}
        onOpenAudit={onOpenAudit}
        outputBundle={outputBundle}
        session={taskSession}
        selectedTask={deliveryTask}
      />
    </section>
  );
}

function ExecutePage({
  executeStatusState,
  mcpSessionsState,
  projectRoot,
  selectedTask,
  tasks,
}: {
  executeStatusState: ExecuteStatusState;
  mcpSessionsState: McpSessionsState;
  projectRoot: string | null;
  selectedTask: V1Issue | null;
  tasks: V1Issue[];
}) {
  const status = executeStatusState.status;
  const summary = status?.summary;
  const sessions = useMemo(() => sortMcpSessionsByLatest(mcpSessionsState.sessions), [mcpSessionsState.sessions]);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [sessionLogState, setSessionLogState] = useState<{
    content: string;
    error: string | null;
    loading: boolean;
  }>({
    content: "",
    error: null,
    loading: false,
  });

  useEffect(() => {
    if (!sessions.length) {
      setSelectedSessionId(null);
      return;
    }
    if (!selectedSessionId || !sessions.some((session) => session.sessionId === selectedSessionId)) {
      setSelectedSessionId(sessions[0]?.sessionId ?? null);
    }
  }, [selectedSessionId, sessions]);

  const selectedSession = sessions.find((session) => session.sessionId === selectedSessionId) ?? sessions[0] ?? null;
  const focusedTask = selectedSession
    ? tasks.find((task) => task.id === selectedSession.issueId) ?? null
    : selectedTask;
  const taskSession = useMemo(
    () => (focusedTask ? pickLatestMcpSessionForIssue(sessions, focusedTask.id) : null),
    [focusedTask, sessions],
  );
  const runCount = sessions.length || summary?.runs || 0;
  const hasRuns = sessions.length > 0;
  const badgeStatus = executeWorkspaceStatusTone(status?.status, executeStatusState.error);
  const badgeLabel = executeWorkspaceStatusLabel(status?.status, executeStatusState.source, executeStatusState.error);
  const taskExecuteItems = useMemo(
    () =>
      focusedTask
        ? taskExecuteSummaryItems(focusedTask, taskSession, mcpSessionsState, executeStatusState)
        : ["先在任务页选择一个任务，再查看它的执行摘要。"],
    [executeStatusState, focusedTask, mcpSessionsState, taskSession],
  );
  const taskStageItems = useMemo(
    () =>
      focusedTask
        ? taskCurrentStageOutputItems(focusedTask, taskSession, null, null, null)
        : ["当前没有选中的任务。"],
    [focusedTask, taskSession],
  );

  useEffect(() => {
    if (!projectRoot || !selectedSession) {
      setSessionLogState({
        content: "",
        error: null,
        loading: false,
      });
      return;
    }
    if (mcpSessionsState.source === "preview") {
      setSessionLogState({
        content: "浏览器预览的模拟会话不会写入真实日志。",
        error: null,
        loading: false,
      });
      return;
    }

    let cancelled = false;
    let timer: number | null = null;

    const loadLogs = async () => {
      try {
        const chunk = await invoke<McpLogChunk>("load_mcp_session_log_chunk", {
          projectRoot,
          sessionId: selectedSession.sessionId,
          cursor: null,
        });
        if (cancelled) {
          return;
        }
        setSessionLogState({
          content: chunk.lines.join("\n"),
          error: null,
          loading: false,
        });
        if (mcpSessionNeedsPolling(selectedSession.status)) {
          timer = window.setTimeout(() => {
            void loadLogs();
          }, 1500);
        }
      } catch (error) {
        if (cancelled) {
          return;
        }
        const message = error instanceof Error ? error.message : String(error);
        setSessionLogState({
          content: "",
          error: message,
          loading: false,
        });
      }
    };

    setSessionLogState({
      content: "",
      error: null,
      loading: true,
    });
    void loadLogs();

    return () => {
      cancelled = true;
      if (timer) {
        window.clearTimeout(timer);
      }
    };
  }, [projectRoot, selectedSession?.sessionId, selectedSession?.status, mcpSessionsState.source]);

  return (
    <section className="v16-page v16-split-page" data-agentflow-page="execute">
      <aside className="v16-list-pane" aria-label="执行列表">
        <header>
          <h2>执行列表</h2>
          <span>{runCount} 项</span>
        </header>
        {hasRuns ? (
          <div className="v16-list-items">
            {sessions.map((session) => (
              <button
                className={session.sessionId === selectedSession?.sessionId ? "v16-list-item active" : "v16-list-item"}
                key={session.sessionId}
                onClick={() => setSelectedSessionId(session.sessionId)}
                title={`${session.issueId} ${mcpSessionStatusLabelZh(session.status)}`}
                type="button"
              >
                <span className="v16-list-item-main">
                  <strong>{session.issueId}</strong>
                  <span>{mcpProviderLabel(session.provider)}</span>
                </span>
                <small>{mcpSessionStatusLabelZh(session.status)}</small>
                <time>{formatTimestamp(session.updatedAt)}</time>
              </button>
            ))}
          </div>
        ) : (
          <div className="v16-list-items">
            <p className="v16-empty-text v16-list-empty-state">还没有执行会话。</p>
          </div>
        )}
      </aside>
      <section className={hasRuns || status ? "v16-detail-pane" : "v16-detail-pane v16-empty-detail-pane"} aria-label="执行详情">
        <header>
          <h2>{focusedTask ? `执行摘要：${focusedTask.id}` : selectedSession ? `执行会话：${selectedSession.issueId}` : "还没有执行会话"}</h2>
          <StatusBadge
            status={
              focusedTask
                ? statusChipForDisplayStatus(focusedTask.displayStatus)
                : selectedSession
                  ? mcpSessionStatusTone(selectedSession.status)
                  : badgeStatus
            }
          >
            {focusedTask ? displayStatusLabelZh(focusedTask.displayStatus) : selectedSession ? mcpSessionStatusLabelZh(selectedSession.status) : badgeLabel}
          </StatusBadge>
        </header>
        <div className="v16-detail-document">
          <SectionList
            title="当前任务"
            items={
              focusedTask
                ? [
                    `${focusedTask.id} · ${focusedTask.title}`,
                    `状态：${displayStatusLabelZh(focusedTask.displayStatus)}`,
                    `角色：${agentRoleLabelZh(focusedTask.requiredAgentRole)}`,
                  ]
                : ["先在任务页选择一个任务。"]
            }
          />
          <SectionList title="执行摘要" items={taskExecuteItems} />
          <SectionList title="当前阶段输出" items={taskStageItems} />
          <SectionList
            title="当前状态"
            items={[
              selectedSession ? `状态：${mcpSessionStatusLabelZh(selectedSession.status)}` : `状态：${badgeLabel}`,
              selectedSession ? `平台：${mcpProviderLabel(selectedSession.provider)}` : "还没有会话被拉起。",
              selectedSession?.lastError ?? executeStatusState.error
                ? `错误：${selectedSession?.lastError ?? executeStatusState.error}`
                : "没有执行错误。",
            ]}
          />
          <SectionList
            title="会话信息"
            items={selectedSession ? executeSessionItems(selectedSession) : ["等待 Project Loop 拉起执行会话。"]}
          />
          {selectedSession ? (
            <CopyableCodeBlock
              content={
                sessionLogState.error
                  ? `读取失败：${sessionLogState.error}`
                  : sessionLogState.loading && !sessionLogState.content
                    ? "正在读取执行日志..."
                    : sessionLogState.content || "当前会话还没有输出日志。"
              }
              language="log"
              maxHeight={260}
              title="执行日志"
            />
          ) : null}
          <SectionList
            title="工作区摘要"
            items={[
              `执行会话：${sessions.length}`,
              `运行中：${summary?.activeRuns ?? sessions.filter((session) => session.status === "running").length}`,
              `已完成：${summary?.completedRuns ?? sessions.filter((session) => session.status === "done").length}`,
              `阻断：${summary?.blockedRuns ?? sessions.filter((session) => ["failed", "cancelled"].includes(session.status)).length}`,
            ]}
          />
          <SectionList
            title="提醒"
            items={
              mcpSessionsState.error
                ? [mcpSessionsState.error]
                : status?.warnings.length
                  ? status.warnings
                  : ["这里只读展示执行会话和执行工作区摘要。真实执行仍由执行助手按任务流程完成。"]
            }
          />
        </div>
      </section>
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
  outputBundle,
  session,
  selectedTask,
}: {
  audits: AuditIndexEntry[];
  delivery: OutputIndexEntry | null;
  evidence: OutputIndexEntry[];
  onOpenAudit: () => void;
  outputBundle: OutputBundleState;
  session: McpSessionSnapshot | null;
  selectedTask: V1Issue | null;
}) {
  const deliveryAudit = delivery ? findAuditForDelivery(audits, delivery.runId) : null;
  const auditDisplay = deliveryAuditStatus(delivery, deliveryAudit);
  const taskEvidence = useMemo(
    () => findEvidenceEntryForTask(evidence, selectedTask),
    [evidence, selectedTask],
  );
  const deliveryProjection = useMemo(
    () =>
      selectedTask
        ? buildTaskDeliveryProjection({ audit: deliveryAudit, delivery, evidence: taskEvidence, session, task: selectedTask })
        : null,
    [delivery, deliveryAudit, selectedTask, session, taskEvidence],
  );
  const deliveryArtifacts = useMemo(
    () => outputBundle.deliveryArtifacts[delivery?.runId ?? selectedTask?.latestRunId ?? ""] ?? null,
    [delivery?.runId, outputBundle.deliveryArtifacts, selectedTask?.latestRunId],
  );
  return (
    <section className={delivery || selectedTask ? "v16-detail-pane" : "v16-detail-pane v16-empty-detail-pane"} aria-label="交付详情">
      <header>
        <h2>{delivery ? `交付包：${deliveryDisplayId(delivery.runId)}` : selectedTask ? `交付摘要：${selectedTask.id}` : "还没有交付材料"}</h2>
        <StatusBadge status={delivery ? "ready" : selectedTask ? statusChipForDisplayStatus(selectedTask.displayStatus) : "idle"}>
          {delivery ? artifactStatusLabel(delivery.status) : selectedTask ? displayStatusLabelZh(selectedTask.displayStatus) : "等待写回"}
        </StatusBadge>
      </header>
      <div className="v16-detail-document">
        <SectionList
          title="当前任务"
          items={
            selectedTask
              ? [
                  `${selectedTask.id} · ${selectedTask.title}`,
                  `状态：${displayStatusLabelZh(selectedTask.displayStatus)}`,
                  `角色：${agentRoleLabelZh(selectedTask.requiredAgentRole)}`,
                ]
              : ["当前没有选中的任务。"]
          }
        />
        <SectionList
          title="交付摘要"
          items={
            deliveryProjection?.summaryItems ??
            (delivery ? [`交付包：${deliveryDisplayId(delivery.runId)}`, `关联任务：${delivery.issueId || "未记录"}`] : ["先在任务页选择一个任务，或在左侧选择一个交付包。"])
          }
        />
        <SectionList
          title="交付空态"
          items={deliveryProjection ? (deliveryProjection.missingItems.length ? deliveryProjection.missingItems : ["交付摘要没有缺失项。"]) : ["未绑定任务，无法判断任务状态所需交付项。"]}
        />
        <SectionList title="评审信息" items={deliveryReviewItems(selectedTask, deliveryArtifacts, session)} />
        <SectionList title="交付说明" items={deliveryReleaseNoteItems(selectedTask, deliveryArtifacts)} />
        <SectionList
          title="最终交付"
          items={deliveryProjection?.packageItems ?? (delivery ? [`交付包：${deliveryDisplayId(delivery.runId)}`, `关联任务：${delivery.issueId || "未记录"}`] : ["还没有交付包。"])}
        />
        <SectionList
          title="关联记录"
          items={[
            delivery?.issueId ? `关联任务：${delivery.issueId}` : "关联任务：未记录",
            delivery?.sourceSpecId ? "关联规格：已确认规格" : "关联规格：未记录",
          ]}
        />
        <SectionList title="验证结果" items={[delivery ? `状态：${artifactStatusLabel(delivery.status)}` : "等待写回。"]} />
        <SectionList title="审计状态" items={[auditDisplay.detail]} />
        <SectionList
          id="v16-delivery-evidence"
          title="证据"
          items={evidence.length ? evidence.map((item) => `${deliveryDisplayId(item.runId)} · ${artifactStatusLabel(item.status)}`) : ["暂无证据。"]}
        />
        <SectionList title="提醒" items={["普通页面默认只展示摘要；原始路径和 JSON 在高级页查看。"]} />
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
  return (
    <section className={selectedAudit || report ? "v16-detail-pane" : "v16-detail-pane v16-empty-detail-pane"} aria-label="审计报告详情">
      <header>
        <h2>{selectedAudit?.auditId ?? report?.audit.auditId ?? "未登记审计"}</h2>
        <StatusBadge status={selectedAudit || report ? "warning" : "idle"}>
          {artifactStatusLabel(selectedAudit?.status ?? report?.audit.status ?? "未登记")}
        </StatusBadge>
      </header>
      <div className="v16-detail-document">
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

type AdvancedJsonFile = {
  description: string;
  displayName?: string;
  error?: string | null;
  loading?: boolean;
  name: string;
  value?: unknown;
};

type AdvancedCategory = {
  files: AdvancedJsonFile[];
  id: string;
  label: string;
  value: unknown;
};

type AdvancedStateJsonFilesState = {
  errors: Record<string, string>;
  files: Record<string, unknown>;
  source: DataSource;
};

const advancedStateFileNames = [
  "workflow.json",
  "gates.json",
  "blockers.json",
  "locks.json",
  "sessions.json",
  "next-actions.json",
] as const;

const initialAdvancedStateJsonFilesState: AdvancedStateJsonFilesState = {
  errors: {},
  files: {},
  source: "idle",
};

function useAdvancedStateJsonFiles(
  projectRoot: string | null,
  stateStatus: StateStatusSnapshot | null,
): AdvancedStateJsonFilesState {
  const [stateJsonFilesState, setStateJsonFilesState] =
    useState<AdvancedStateJsonFilesState>(initialAdvancedStateJsonFilesState);

  useEffect(() => {
    if (!projectRoot) {
      setStateJsonFilesState(initialAdvancedStateJsonFilesState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setStateJsonFilesState({
        errors: {},
        files: buildBrowserPreviewAdvancedStateFiles(projectRoot, stateStatus),
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setStateJsonFilesState((current) => ({ ...current, errors: {}, source: "loading" }));

    void loadAdvancedStateFiles(projectRoot, stateStatus)
      .then((nextState) => {
        if (!cancelled) {
          setStateJsonFilesState(nextState);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : String(error);
          setStateJsonFilesState({
            errors: Object.fromEntries(advancedStateFileNames.map((name) => [name, message])),
            files: {},
            source: "unavailable",
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, stateStatus?.updatedAt]);

  return stateJsonFilesState;
}

async function loadAdvancedStateFiles(
  projectRoot: string,
  stateStatus: StateStatusSnapshot | null,
): Promise<AdvancedStateJsonFilesState> {
  const loaders: Array<[string, Promise<unknown>]> = [
    ["workflow.json", stateStatus ? Promise.resolve(stateStatus) : invoke("load_state_status", { projectRoot })],
    ["gates.json", invoke("load_workflow_gates", { projectRoot })],
    ["blockers.json", invoke("load_blockers", { projectRoot })],
    ["locks.json", invoke("load_state_locks", { projectRoot })],
    [
      "sessions.json",
      invoke<Record<string, unknown>>("load_state_index", { projectRoot }).then((index) => ({
        version: "state-sessions.reader.v1",
        sessions: Array.isArray(index.sessions) ? index.sessions : [],
      })),
    ],
    ["next-actions.json", invoke("load_next_actions", { projectRoot })],
  ];
  const entries = await Promise.all(
    loaders.map(async ([name, loader]) => {
      try {
        return { name, status: "fulfilled" as const, value: await loader };
      } catch (error) {
        return {
          error: error instanceof Error ? error.message : String(error),
          name,
          status: "rejected" as const,
        };
      }
    }),
  );

  const files: Record<string, unknown> = {};
  const errors: Record<string, string> = {};
  entries.forEach((entry) => {
    if (entry.status === "fulfilled") {
      files[entry.name] = entry.value;
    } else {
      errors[entry.name] = entry.error;
    }
  });

  return {
    errors,
    files,
    source: Object.keys(errors).length ? "unavailable" : "tauri",
  };
}

function buildBrowserPreviewAdvancedStateFiles(
  projectRoot: string,
  stateStatus: StateStatusSnapshot | null,
): Record<string, unknown> {
  const workflow = stateStatus ?? {
    version: "state-status.browser-preview",
    projectRoot,
    status: "ready",
    currentStage: "workspace-ready",
    auditStatus: "not-requested",
    activeIssueId: null,
    activeRunId: null,
    health: {},
    nextActions: [],
    blockers: [],
    updatedAt: 0,
  };

  return {
    "blockers.json": {
      version: "workflow-blockers.browser-preview",
      blockers: workflow.blockers ?? [],
    },
    "gates.json": {
      version: "workflow-gates.browser-preview",
      allowedNextActions: workflow.nextActions ?? [],
      auditStatus: workflow.auditStatus,
      blockers: workflow.blockers ?? [],
      currentStage: workflow.currentStage,
      health: workflow.health ?? {},
    },
    "locks.json": {
      version: "state-locks.browser-preview",
      active: [],
      cleanupCandidates: [],
      stale: [],
    },
    "next-actions.json": {
      version: "workflow-next-actions.browser-preview",
      actions: (workflow.nextActions ?? []).map((action) => ({
        action,
        label: buildNextActionLabel(action),
      })),
    },
    "sessions.json": {
      version: "state-sessions.browser-preview",
      sessions: [],
    },
    "workflow.json": workflow,
  };
}

function AdvancedPage({
  agentManualState,
  inputSnapshotState,
  issueStatusIndexState,
  outputBundle,
  projectRoot,
  projectFilesState,
  projectPanelState,
  initializationState,
  stateStatusState,
  workspaceData,
}: {
  agentManualState: unknown;
  inputSnapshotState: InputSnapshotState;
  issueStatusIndexState: IssueStatusIndexState;
  outputBundle: OutputBundleState;
  projectRoot: string | null;
  projectFilesState: ProjectFilesState;
  projectPanelState: ProjectPanelState;
  initializationState: ProjectInitializationState;
  stateStatusState: StateStatusState;
  workspaceData: WorkspaceDataState;
}) {
  const stateJsonFilesState = useAdvancedStateJsonFiles(projectRoot, stateStatusState.status);
  const categories = [
    { id: "state", label: "状态", value: stateStatusState, files: advancedFilesForCategory("state", stateStatusState, stateJsonFilesState) },
    { id: "agentRoles", label: "Agent 角色", value: agentRoleRulesDocument(), files: advancedFilesForCategory("agentRoles", agentRoleRulesDocument()) },
    { id: "initialization", label: "初始化", value: initializationState, files: advancedFilesForCategory("initialization", initializationState) },
    { id: "panel", label: "Panel", value: projectPanelState, files: advancedFilesForCategory("panel", projectPanelState) },
    { id: "spec", label: "Spec", value: inputSnapshotState, files: advancedFilesForCategory("spec", inputSnapshotState) },
    { id: "tasks", label: "任务投影", value: issueStatusIndexState, files: advancedFilesForCategory("tasks", issueStatusIndexState) },
    { id: "audit", label: "Audit", value: outputBundle.auditReport, files: advancedFilesForCategory("audit", outputBundle.auditReport) },
    { id: "settings", label: "设置", value: { agentManualState, projectFilesState, workspaceData }, files: advancedFilesForCategory("settings", { agentManualState, projectFilesState, workspaceData }) },
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
  categories: AdvancedCategory[];
  onSelectCategory: (categoryId: string) => void;
  selectedCategory: AdvancedCategory;
}) {
  const [selectedFileName, setSelectedFileName] = useState<string | null>(selectedCategory.files[0]?.name ?? null);
  const selectedFile =
    selectedCategory.files.find((file) => file.name === selectedFileName) ?? selectedCategory.files[0] ?? null;

  useEffect(() => {
    setSelectedFileName(selectedCategory.files[0]?.name ?? null);
  }, [selectedCategory.id, selectedCategory.files[0]?.name]);

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
          {selectedCategory.files.length ? selectedCategory.files.map((file) => (
            <button
              aria-current={file.name === selectedFile?.name ? "true" : undefined}
              className={file.name === selectedFile?.name ? "active" : ""}
              key={file.name}
              onClick={() => setSelectedFileName(file.name)}
              type="button"
            >
              <strong>{file.displayName ?? file.name}</strong>
              <span>{file.description}</span>
            </button>
          )) : <p className="v16-empty-text">没有可展示的 JSON 文件。</p>}
        </div>
      </section>
      <section className="v16-advanced-reader">
        <header>
          <h2>{selectedFile?.name ?? "JSON Reader"}</h2>
          <p>{selectedFile?.description ?? "没有可展示的 JSON 文件。"} 只读展示，不编辑 JSON，不修复状态，不清理锁，不触发审计。</p>
        </header>
        <JsonReader
          emptyMessage={selectedFile ? "这个 JSON 文件暂无可展示内容。" : "没有可展示的 JSON 文件。"}
          error={selectedFile?.error ?? null}
          loading={selectedFile?.loading ?? false}
          value={selectedFile?.value ?? selectedCategory.value}
        />
      </section>
    </div>
  );
}

function JsonReader({
  emptyMessage = "没有可展示的 JSON 内容。",
  error,
  loading,
  value,
}: {
  emptyMessage?: string;
  error?: string | null;
  loading?: boolean;
  value: unknown;
}) {
  let content = "";
  if (loading) {
    content = "正在读取 JSON 文件。";
  } else if (error) {
    content = `读取失败：${error}`;
  } else if (value === null || value === undefined) {
    content = emptyMessage;
  } else {
    content = JSON.stringify(value, null, 2);
  }

  return (
    <pre className="v16-json-reader" aria-label="JSON Reader">
      <code>{content}</code>
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
            ? "等待写回。请确认任务包已经执行，并刷新任务状态流。"
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
  { id: "backlog", label: "待处理" },
  { id: "todo", label: "准备开工" },
  { id: "in_progress", label: "正在做" },
  { id: "in_review", label: "正在评审" },
  { id: "done", label: "已完成" },
  { id: "blocked", label: "已阻断" },
  { id: "cancel", label: "已取消" },
];

const displayStatusOrder = new Map(displayStatusColumns.map((column, index) => [column.id, index]));

const buildAgentPipelineStageIds = [
  "issue-preflight",
  "test-design",
  "implement",
  "sandbox-verify",
  "create-pr",
  "merge-pr",
  "writeback-done",
] as const;

function defaultBuildAgentExecutionPipeline(): ExecutionPipeline {
  return {
    agentRole: "build-agent",
    gitProviders: [],
    mergeModes: ["auto-merge-if-eligible", "manual-merge"],
    stages: [
      {
        evidence: [
          "AgentFlow spec issue is the only active task source; executionPipeline is read from that issue contract",
          "no external issue/task/plan/queue/thread/tool state is used as task authority",
          "spec issue status is backlog before preflight",
          "blockedBy dependencies are done",
          "Panel Context Pack exists or is generated",
          "current run is created by AgentFlow official runtime entrypoint before source edits",
          "no `.agentflow/**` facts are handwritten; official AgentFlow loop commands are used instead",
          "working tree has no uncommitted user source changes before in_progress",
          "spec issue status changed to todo after preflight",
        ],
        goal:
          "只认当前 AgentFlow spec issue。确认 issue 仍在 backlog，依赖已完成、合同完整、Context Pack 可读或可补生成、工作区干净；随后通过 AgentFlow 官方 run loop / runtime preflight 创建当前 run。preflight 通过后先把 issue 切到 todo，再准备进入 in_progress。禁止手写 `.agentflow/**` 只表示不能直接改事实文件，不是禁止调用 AgentFlow 官方命令推进 loop。GitHub/GitLab 不在这个阶段检测。",
        label: "执行前置检测",
        required: true,
        stageId: "issue-preflight",
      },
      {
        evidence: [
          "test points derived from SPEC and issue",
          "failing test result or TDD-not-applicable reason",
          "planned sandbox validation commands",
        ],
        goal: "从 SPEC 和当前 issue 推导测试点。能做 TDD 就先补失败测试；不能做 TDD 就记录原因，并给出替代验证方式。",
        label: "测试设计",
        required: true,
        stageId: "test-design",
      },
      {
        evidence: ["git diff --stat", "changed-files summary"],
        goal: "按测试设计和 issue 合同，在 allowedPaths 内完成代码、配置或测试改动。",
        label: "Agent 执行 issue",
        required: true,
        stageId: "implement",
      },
      {
        evidence: ["validation command records", "browser smoke evidence when applicable", "git diff --check"],
        goal: "在本地受控沙箱中运行验证命令，并收集 stdout、stderr、exit code 以及浏览器或截图证据。",
        label: "沙箱验证",
        required: true,
        stageId: "sandbox-verify",
      },
      {
        evidence: [
          "PR/MR URL",
          "AgentFlow Build Agent PR/MR template completed",
          "PR/MR body validation summary",
          "draft or ready state",
        ],
        goal: "推送任务分支，按 AgentFlow Build Agent PR/MR 模板创建 GitHub PR 或 GitLab MR，并写入任务、范围、验证结果、影响、回滚和 review gate；如果 mergeMode 是 auto-merge-if-eligible，不能停在 Draft PR/MR。",
        label: "创建 PR/MR",
        required: true,
        stageId: "create-pr",
      },
      {
        evidence: [
          "merge mode",
          "GitHub path: gh pr ready result and gh pr merge --auto result",
          "GitLab path: glab mr update --ready result and glab mr merge --auto-merge result",
          "auto-merge rejection reason when falling back to manual-merge",
          "in_review wait evidence when manual-merge fallback is active",
          "merge commit or merged PR/MR state",
        ],
        goal: "默认先走 auto-merge-if-eligible：PR/MR ready 后按 provider 自动合并，并轮询到 merged；如果自动合并条件不满足，就回落到 manual-merge，issue 保持 in_review，等待人合并，再由本地检测确认 merged 后继续。",
        label: "合并 PR/MR",
        required: true,
        stageId: "merge-pr",
      },
      {
        evidence: [
          "target/release/agentflow build-agent complete --request <completion-request.json> after cargo build --release --bin agentflow",
          "or target/debug/agentflow build-agent complete --request <completion-request.json>",
          "issue status done",
        ],
        goal: "PR/MR 合并后，用预检确认过的新 AgentFlow CLI 调用 build-agent complete，写回 run、evidence、delivery 和任务 Done 状态。",
        label: "写回 Done",
        required: true,
        stageId: "writeback-done",
      },
    ],
    version: "build-agent-execution-pipeline.v1",
  };
}

function buildTaskItems(inputIssues: InputIssue[] | null, issueStatusIndex: IssueStatusIndex | null): V1Issue[] {
  if (inputIssues) {
    return sortTasksByExecutionOrder(inputIssues.map((issue) => inputIssueToV1Issue(issue, issueStatusIndex)));
  }
  return [];
}

function inputIssueToV1Issue(issue: InputIssue, issueStatusIndex: IssueStatusIndex | null): V1Issue {
  const indexed = issueStatusIndex?.issues.find((item) => item.issueId === issue.issueId);
  const displayStatus = indexed?.displayStatus ?? issue.displayStatus;
  const issueCategory = issue.issueCategory ?? "spec";
  const requiredAgentRole = issue.requiredAgentRole ?? (issueCategory === "audit" ? "audit-agent" : "build-agent");
  const auditId = issue.audit?.auditId ?? (issueCategory === "audit" ? issue.issueId : null);
  const auditOutputDir = issue.audit?.auditOutputDir ?? (auditId ? `.agentflow/audit/${auditId}` : null);
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
    auditStatus: indexed?.auditStatus ?? "not-requested",
    boundary: issue.nonGoals,
    codexInstructions: issue.validationHints,
    dependencies: issue.relations?.blockedBy ?? [],
    deliveryStatus: indexed?.deliveryStatus ?? "missing",
    displayStatus,
    evidenceStatus: indexed?.evidenceStatus ?? "missing",
    evidenceRequired: issue.acceptanceCriteria,
    executeStatus: indexed?.executeStatus ?? null,
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
    issuePath: issue.issuePath ?? issue.system?.path ?? `.agentflow/spec/issues/${issue.issueId}.json`,
    latestRunId: indexed?.latestRunId ?? null,
    milestoneId: null,
    nonGoals: issue.nonGoals,
    projectId: issue.projectId ?? null,
    priority: indexed?.priority ?? issue.priority ?? "p2",
    rawStatus: issue.status,
    requiredAgentRole,
    executionRisk: issue.executionRisk || indexed?.executionRisk || "low",
    sourceDeliveryPath: issue.audit?.sourceDeliveryPath ?? null,
    sourceReleaseId: issue.audit?.sourceReleaseId ?? null,
    sourceSpecId: issue.sourceSpecId ?? null,
    sourceSpecPath:
      issue.sourceSpecPath ??
      (issue.sourceSpecId ? `docs/requirements/${issue.sourceSpecId}.md` : null),
    scope: issue.scope,
    status: issue.status,
    title: issue.title,
    updatedAt: issue.system?.updatedAt ?? issue.system?.createdAt ?? null,
    validationCommands: issue.validationCommands?.length ? issue.validationCommands : issue.validationHints,
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
    const outputDir = auditOutputDir || (auditId ? `.agentflow/audit/${auditId}` : "");
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
    evidencePath: `.agentflow/tasks/${issueId}/evidence/evidence.json`,
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
    return [".agentflow/tasks/*/runs/**", ".agentflow/tasks/*/evidence/**"];
  }
  return [".agentflow/audit/**", ".agentflow/spec/**"];
}

function defaultForbiddenActions(issueCategory?: string | null) {
  if (issueCategory === "audit") {
    return ["process-spec-issue", "write-source-code", "execute-project-commands", "generate-public-delivery-record"];
  }
  return ["process-audit-issue", "write-audit-report", "write-audit-findings"];
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

function sortMcpSessionsByLatest(sessions: McpSessionSnapshot[]) {
  return [...sessions].sort(
    (left, right) => right.updatedAt - left.updatedAt || right.sessionId.localeCompare(left.sessionId),
  );
}

function mcpSessionNeedsPolling(status?: McpSessionSnapshot["status"] | null) {
  return status ? ["queued", "claimed", "starting", "running", "in-review"].includes(status) : false;
}

function pickLatestMcpSessionForIssue(sessions: McpSessionSnapshot[], issueId?: string | null) {
  if (!issueId) {
    return null;
  }
  return sortMcpSessionsByLatest(sessions).find((session) => session.issueId === issueId) ?? null;
}

function statusChipForDisplayStatus(status: IssueDisplayStatus = "backlog"): StatusChipStatus {
  const chips: Record<IssueDisplayStatus, StatusChipStatus> = {
    backlog: "idle",
    blocked: "blocked",
    cancel: "blocked",
    done: "done",
    in_progress: "working",
    in_review: "warning",
    todo: "ready",
  };
  return chips[status];
}

function taskMenuStatusSummary(task: {
  displayStatus?: IssueDisplayStatus;
  blockedBy?: string[];
  dependencies?: string[];
  deliveryStatus?: string | null;
}) {
  const status = task.displayStatus ?? "backlog";
  const blockerCount = task.blockedBy?.length ?? task.dependencies?.length ?? 0;

  const summaries: Record<IssueDisplayStatus, string> = {
    backlog: "还没有进入执行",
    blocked: blockerCount ? `等待 ${blockerCount} 个阻断项解除` : "等待解除阻断",
    cancel: "",
    done: "",
    in_progress: "正在实现并做本地验证",
    in_review: "等待合并与最终写回",
    todo: "前置检测已通过，等待开工",
  };
  return summaries[status];
}

function projectMenuWorkflowStatus(group: TaskProjectGroup): IssueDisplayStatus {
  if (!group.issues.length) {
    return "backlog";
  }

  const cancelledIssues = group.issues.filter((issue) => issue.displayStatus === "cancel");
  if (cancelledIssues.length === group.issues.length) {
    return "cancel";
  }

  const finishedIssues = group.issues.filter((issue) => issue.displayStatus === "done" || issue.displayStatus === "cancel");
  if (finishedIssues.length === group.issues.length) {
    return "done";
  }

  if (group.issues.some((issue) => issue.displayStatus === "in_progress")) {
    return "in_progress";
  }
  if (group.issues.some((issue) => issue.displayStatus === "in_review")) {
    return "in_review";
  }
  if (group.issues.some((issue) => issue.displayStatus === "todo")) {
    return "todo";
  }

  const unfinishedIssues = group.issues.filter((issue) => issue.displayStatus !== "done" && issue.displayStatus !== "cancel");
  if (unfinishedIssues.length && unfinishedIssues.every((issue) => issue.displayStatus === "blocked" || issue.status === "blocked")) {
    return "blocked";
  }

  return "backlog";
}

function taskProjectTimelineSections(issues: TaskIssueNode[]) {
  const past = issues.filter((issue) => issue.displayStatus === "done" || issue.displayStatus === "cancel");
  const current = issues.filter((issue) =>
    ["todo", "in_progress", "in_review", "blocked"].includes(issue.displayStatus),
  );
  const futurePool = issues.filter((issue) => issue.displayStatus === "backlog");

  const currentIssues = current.length ? current : futurePool.slice(0, 1);
  const futureIssues = current.length ? futurePool : futurePool.slice(1);

  return [
    {
      id: "current" as const,
      issues: currentIssues,
    },
    {
      id: "past" as const,
      issues: past,
    },
    {
      id: "future" as const,
      issues: futureIssues,
    },
  ].filter((section) => section.issues.length);
}

type TaskTimelineTone = "current" | "future" | "done" | "cancel";

function taskTimelineToneForIssue(issue: { displayStatus?: IssueDisplayStatus }): TaskTimelineTone {
  switch (issue.displayStatus) {
    case "done":
      return "done";
    case "cancel":
      return "cancel";
    case "backlog":
      return "future";
    default:
      return "current";
  }
}

type ProjectDisplayStatus = "planned" | "active" | "blocked" | "done" | "canceled";
type ProjectBrainDisplayStatus =
  | "not-initialized"
  | "needs-goal"
  | "needs-plan"
  | "needs-confirmation"
  | "ready-for-project-loop"
  | "needs-recheck"
  | "blocked";

function projectDisplayStatusForGroup(group: TaskProjectGroup): ProjectDisplayStatus {
  if (!group.issues.length) {
    return normalizeProjectDisplayStatus(group.status);
  }

  const finishedIssues = group.issues.filter((issue) => issue.displayStatus === "done" || issue.displayStatus === "cancel");
  if (finishedIssues.length === group.issues.length) {
    return group.issues.every((issue) => issue.displayStatus === "cancel") ? "canceled" : "done";
  }

  const unfinishedIssues = group.issues.filter((issue) => issue.displayStatus !== "done" && issue.displayStatus !== "cancel");
  if (unfinishedIssues.length && unfinishedIssues.every((issue) => issue.displayStatus === "blocked" || issue.status === "blocked")) {
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

function projectBrainStatusLabelZh(status?: string | null) {
  const labels: Record<ProjectBrainDisplayStatus, string> = {
    "blocked": "有阻断",
    "needs-confirmation": "待确认",
    "needs-goal": "缺目标",
    "needs-plan": "缺计划",
    "needs-recheck": "待复查",
    "not-initialized": "未初始化",
    "ready-for-project-loop": "可进入项目循环",
  };
  return labels[(status ?? "not-initialized") as ProjectBrainDisplayStatus] ?? "未初始化";
}

function projectBrainDocumentStatusLabelZh(status?: string | null) {
  const labels: Record<string, string> = {
    blocked: "有阻断",
    confirmed: "已确认",
    draft: "草稿",
    missing: "缺失",
    "needs-confirmation": "待确认",
    stale: "已过期",
  };
  return labels[(status ?? "missing").toLowerCase()] ?? "缺失";
}

function projectBrainActionLabelZh(action?: string | null, fallbackLabel?: string | null) {
  if (fallbackLabel?.trim()) {
    return fallbackLabel.trim();
  }
  const labels: Record<string, string> = {
    "confirm-project-brain": "确认 Project Brain",
    "create-goal-draft-preview": "生成 Goal 草稿预览",
    "create-plan-draft-preview": "生成 Plan 草稿预览",
    "resolve-project-brain-blocker": "处理 Project Brain 阻断",
    "run-goal-recheck": "重新检查项目目标",
    "start-project-loop": "进入项目循环",
  };
  return labels[action ?? ""] ?? "等待下一步";
}

function projectCompletionStateLabelZh(state?: string | null, outcome?: string | null) {
  const normalizedState = (state ?? "").toLowerCase();
  const normalizedOutcome = (outcome ?? "").toLowerCase();
  if (normalizedState === "accepted" || normalizedOutcome === "accept") {
    return "已接受";
  }
  const labels: Record<string, string> = {
    "goal-recheck": "待完成判断",
    "continue": "继续推进",
    "adjust": "需要调整",
    "pause": "已暂停",
    "next-stage": "进入下一阶段",
  };
  return labels[normalizedState] ?? (state ? state : "未开始");
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
  if (stateWorkspaceBlocked(stateStatusState.status)) {
    return "blocked";
  }
  return "ready";
}

function stateWorkspaceBlocked(status?: StateStatusSnapshot | null) {
  if (!status) {
    return false;
  }
  return status.status === "blocked" || status.currentStage === "workspace-blocked" || status.currentStage === "execute-blocked";
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
    return "你现在是 Audit Agent，只能执行 audit issue。如果你不是 audit-agent，请停止执行。不要修改源码、不要生成 patch、不要创建远程 PR/MR。";
  }
  return "你现在是 Build Agent，只能执行 spec issue。如果你不是 build-agent，请停止执行。AgentFlow 当前 spec issue 是唯一任务源；handoff package 只是这份 issue 的派生快照；executionPipeline 只是这份 issue 合同里的一部分，不是独立任务源。不要把外部 issue、任务、计划、队列、线程或工具状态当成任务源，也不要用外部状态拆分、重排或推进 AgentFlow 任务。按执行前置检测、测试设计、执行、沙箱验证、创建 PR/MR、合并 PR/MR、写回 Done 的流程执行。先调用 `agentflow build-agent start --issue-id <issue-id>` 创建当前 run；没有 run 就不能开工。完成本地验证后，调用 `agentflow build-agent prepare-review --request <completion-request.json>` 生成 result、evidence、delivery，并把 issue 推到 in_review。PR/MR 创建或合并后，调用 `agentflow build-agent write-merge-proof --issue-id <issue-id> --run-id <run-id> --provider <github|gitlab> --merge-mode <auto-merge-if-eligible|manual-merge> [--remote-url <url>] [--merged]` 记录 review 状态。PR/MR 合并后，再调用 `agentflow build-agent complete --request <completion-request.json>` 写回 Done。不要手写 `.agentflow/**`，但这不等于不能调用 AgentFlow 官方命令推进 loop、补生成 Context Pack 或写回完成结果。写回 Done 前必须确认当前 AgentFlow CLI 支持这些 build-agent 命令；不要直接复用过期 target/release/agentflow。不要写 audit report、findings、evidence-map 或 traceability。";
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

function taskOutputSummaryItems(task: V1Issue) {
  const entries = Object.entries(task.expectedOutputs ?? {});
  if (!entries.length) {
    return ["还没有登记预期产物。"];
  }
  return entries.map(([key]) => expectedOutputLabelZh(key));
}

function taskOutputItems(task: V1Issue) {
  const entries = Object.entries(task.expectedOutputs ?? {});
  if (!entries.length) {
    return ["未提供输出位置。"];
  }
  return entries.map(([key, value]) => `${key}: ${value}`);
}

function taskPublicDeliveryEntries(tasks: V1Issue[]): OutputIndexEntry[] {
  return sortOutputEntriesByLatest(
    tasks
      .filter((task) => task.displayStatus === "in_review" || task.displayStatus === "done")
      .map((task) => {
        const runId = task.latestRunId ?? `task-${task.id}`;
        return {
          issueId: task.id,
          path: task.displayStatus === "done" ? "CHANGELOG.md" : "PR/MR body",
          runId,
          sourceSpecId: task.sourceSpecId ?? "",
          status: task.displayStatus === "done" ? "published" : "ready",
          updatedAt: task.updatedAt ?? task.createdAt ?? 0,
        };
      }),
  );
}

function findDeliveryEntryForTask(deliveries: OutputIndexEntry[], task: V1Issue | null) {
  if (!task) {
    return null;
  }
  const latestRunId = task.latestRunId ?? null;
  return (
    sortOutputEntriesByLatest(deliveries).find((delivery) =>
      (latestRunId ? delivery.runId === latestRunId : false) || delivery.issueId === task.id,
    ) ?? null
  );
}

function findEvidenceEntryForTask(evidenceEntries: OutputIndexEntry[], task: V1Issue | null) {
  if (!task) {
    return null;
  }
  const latestRunId = task.latestRunId ?? null;
  return (
    sortOutputEntriesByLatest(evidenceEntries).find((evidence) =>
      (latestRunId ? evidence.runId === latestRunId : false) || evidence.issueId === task.id,
    ) ?? null
  );
}

function expectedOutputLabelZh(key: string) {
  const labels: Record<string, string> = {
    "audit-report.md": "审计报告",
    "audit.json": "审计结论",
    "evidence-map.json": "证据映射",
    "findings.json": "审计发现",
    "traceability.json": "追溯关系",
    evidencePath: "验证证据",
  };
  return labels[key] ?? key;
}

function executeProgressLabel(status?: string | null) {
  if (!status) {
    return "还没有进入执行。";
  }
  const labels: Record<string, string> = {
    blocked: "执行被阻断",
    cancelled: "执行已取消",
    checkpointed: "已记录检查点",
    completed: "执行已完成",
    failed: "执行失败",
    in_progress: "正在执行",
    patching: "正在应用改动",
    planned: "前置检测完成，等待正式执行",
    preflight: "正在做前置检测",
    queued: "等待拉起执行",
    running: "正在执行",
    validating: "正在做沙箱验证",
  };
  return labels[status] ?? status;
}

function taskCurrentStageOutputItems(
  task: V1Issue,
  session: McpSessionSnapshot | null,
  delivery: OutputIndexEntry | null,
  evidence: OutputIndexEntry | null,
  audit: AuditIndexEntry | null,
  projection?: TaskProjection | null,
) {
  const publicDelivery = projection?.publicDelivery ?? null;
  const runtime = projection?.runtime ?? null;
  const projectionSession = projection?.session ?? null;
  const projectedDelivery = projection?.delivery ?? null;
  const projectedAudit = projection?.audit ?? null;
  const projectedEvidencePath = projectedDelivery?.evidencePath ?? publicDelivery?.evidencePath ?? null;
  const projectedPrUrl = projectedDelivery?.prUrl ?? publicDelivery?.prUrl ?? null;
  const projectedMergeCommit = projectedDelivery?.mergeCommit ?? publicDelivery?.mergeCommit ?? null;
  const projectedPublicRecord =
    projectedDelivery?.publicRecordPath ?? publicDelivery?.changelogPath ?? publicDelivery?.releaseNotesUrl ?? null;
  const sessionStatus = projectionSession?.status ?? session?.status ?? null;
  const sessionBranchName = projectionSession?.branchName ?? runtime?.branchName ?? session?.branchName ?? null;
  const sessionPlanPath = projectionSession?.planPath ?? null;
  const sessionLogPath = projectionSession?.logPath ?? null;

  switch (task.displayStatus ?? "backlog") {
    case "backlog":
      return [
        "任务合同已生成。",
        task.dependencies.length ? `前置依赖：${task.dependencies.join("、")}` : "当前没有前置依赖。",
        task.contextPackPath ? "Context Pack 已登记，等待进入执行准备。" : "Context Pack 还没有准备到执行阶段。",
      ];
    case "todo":
      return [
        runtime?.runId ?? task.latestRunId ? `当前 Run：${runtime?.runId ?? task.latestRunId}` : "当前 Run：等待创建。",
        task.contextPackPath ? "Context Pack 已可用。" : "Context Pack：等待生成。",
        sessionStatus ? `启动请求：${mcpSessionStatusLabelZh(sessionStatus)}` : null,
        `执行准备：${executeProgressLabel(runtime?.runStatus ?? task.executeStatus)}`,
      ].filter((item): item is string => Boolean(item));
    case "in_progress":
      return [
        runtime?.runId ?? task.latestRunId ? `当前 Run：${runtime?.runId ?? task.latestRunId}` : "当前 Run：未记录。",
        `执行进度：${executeProgressLabel(runtime?.runStatus ?? task.executeStatus)}`,
        sessionStatus
          ? `会话状态：${mcpSessionStatusLabelZh(sessionStatus)}`
          : "会话状态：还没有拉起，或已提前退出。",
        sessionBranchName ? `工作分支：${sessionBranchName}` : null,
        sessionPlanPath ? `执行计划：${sessionPlanPath}` : null,
        sessionLogPath ? `事件日志：${sessionLogPath}` : null,
        ...(runtime?.latestCheckpointSummary ? [`最新检查点：${runtime.latestCheckpointSummary}`] : []),
        ...(session?.lastError ? [`最近错误：${session.lastError}`] : []),
      ].filter((item): item is string => Boolean(item));
    case "in_review":
      return [
        runtime?.runId ?? task.latestRunId ? `当前 Run：${runtime?.runId ?? task.latestRunId}` : "当前 Run：未记录。",
        projectedEvidencePath ? `验证证据：${projectedEvidencePath}` : evidence ? `验证证据：${artifactStatusLabel(evidence.status)}` : "验证证据：等待写回。",
        projectedPrUrl ? "PR/MR：已创建。" : session?.prUrl ? "PR/MR：已创建。" : "PR/MR：等待记录。",
        projectedPublicRecord ? `公开交付：${projectedPublicRecord}` : "公开交付：等待记录。",
        sessionStatus ? `会话状态：${mcpSessionStatusLabelZh(sessionStatus)}` : null,
      ].filter((item): item is string => Boolean(item));
    case "done":
      return [
        runtime?.runId ?? task.latestRunId ? `最终 Run：${runtime?.runId ?? task.latestRunId}` : "最终 Run：未记录。",
        projectedEvidencePath ? `验证证据：${projectedEvidencePath}` : evidence ? `验证证据：${artifactStatusLabel(evidence.status)}` : "验证证据：未找到记录。",
        projectedPrUrl ? `PR/MR：${projectedPrUrl}` : "PR/MR：未找到记录。",
        projectedMergeCommit ? `合并提交：${projectedMergeCommit}` : "合并提交：未找到记录。",
        projectedPublicRecord ? `公开交付：${projectedPublicRecord}` : "公开交付：未找到记录。",
        sessionBranchName ? `工作分支：${sessionBranchName}` : null,
        projectedAudit?.status && projectedAudit.status !== "not-requested"
          ? `后续审计：${artifactStatusLabel(projectedAudit.status)}`
          : audit
            ? `后续审计：${artifactStatusLabel(audit.status)}`
            : "后续审计：独立流程，按需触发。",
      ].filter((item): item is string => Boolean(item));
    case "blocked":
      return [
        "当前任务被关键因素阻断。",
        task.dependencies.length ? `相关依赖：${task.dependencies.join("、")}` : "阻断原因等待补充。",
      ];
    case "cancel":
      return ["当前任务已取消，不再继续执行。"];
    default:
      return ["还没有阶段输出。"];
  }
}

function taskExecuteSummaryItems(
  task: V1Issue,
  session: McpSessionSnapshot | null,
  mcpSessionsState: McpSessionsState,
  executeStatusState: ExecuteStatusState,
) {
  const items = [
    task.latestRunId ? `当前 Run：${task.latestRunId}` : "当前 Run：还没有创建。",
    `执行状态：${executeProgressLabel(task.executeStatus)}`,
  ];

  if (session) {
    items.push(`外部会话：${mcpSessionStatusLabelZh(session.status)}`);
    items.push(`平台：${mcpProviderLabel(session.provider)}`);
    if (session.branchName) {
      items.push(`工作分支：${session.branchName}`);
    }
    if (session.prUrl) {
      items.push("PR/MR：已记录。");
    }
    if (session.lastError) {
      items.push(`最近错误：${session.lastError}`);
    }
    return items;
  }

  if (mcpSessionsState.source === "loading") {
    items.push("执行会话：正在读取。");
  } else if (mcpSessionsState.error) {
    items.push(`执行会话：读取失败，${mcpSessionsState.error}`);
  } else {
    items.push("执行会话：当前没有会话记录。");
  }

  if (executeStatusState.error) {
    items.push(`执行工作区：${executeStatusState.error}`);
  } else if (executeStatusState.status?.warnings.length) {
    items.push(`执行工作区：${executeStatusState.status.warnings[0]}`);
  } else {
    items.push("执行工作区：没有额外告警。");
  }

  return items;
}

function deliveryReviewItems(
  task: V1Issue | null,
  artifact: DeliveryArtifactState | null,
  session: McpSessionSnapshot | null,
  projection?: TaskProjection | null,
) {
  if (!task) {
    return ["先选择一个任务。"];
  }

  const mergeProof = artifact?.mergeProof;
  const prMetadata = artifact?.prMetadata;
  const publicDelivery = projection?.publicDelivery ?? null;
  const projectionSession = projection?.session ?? null;
  const provider = mergeProof?.provider ?? prMetadata?.provider ?? projectionSession?.provider ?? session?.provider ?? null;
  const reviewUrl = publicDelivery?.prUrl ?? mergeProof?.remoteUrl ?? prMetadata?.remotePrUrl ?? session?.prUrl ?? null;
  const mergeMode = mergeProof?.mergeMode ?? prMetadata?.mergeMode ?? null;
  const branchName =
    projection?.branchName ?? projectionSession?.branchName ?? prMetadata?.branchName ?? session?.branchName ?? null;
  const merged = Boolean(publicDelivery?.mergeCommit) || mergeProof?.merged || prMetadata?.merged || session?.mergeState === "merged";

  if (!mergeProof && !prMetadata && !session && !publicDelivery?.prUrl && !publicDelivery?.mergeCommit) {
    return task.displayStatus === "in_review" || task.displayStatus === "done"
      ? ["评审信息缺失：当前状态应该已经创建评审记录，但状态投影还没有 PR/MR 或合并证明。"]
      : ["当前阶段还没有评审信息。"];
  }

  const reviewState = merged
    ? "评审状态：已合并。"
    : prMetadata?.createdRemotePr || reviewUrl
      ? "评审状态：已创建远端评审请求。"
      : prMetadata?.status === "draft-only"
        ? "评审状态：只有本地草稿，还没有远端评审请求。"
        : projectionSession?.status === "in-review" || session?.status === "in-review"
          ? "评审状态：正在评审。"
          : "评审状态：评审信息已写入。";

  return [
    reviewState,
    reviewUrl ? `评审链接：${reviewUrl}` : "评审链接：未记录",
    provider ? `平台：${mcpProviderLabel(provider)}` : "平台：未记录",
    mergeMode ? `合并模式：${mergeMode}` : "合并模式：未记录",
    branchName ? `工作分支：${branchName}` : "工作分支：未记录",
    publicDelivery?.mergeCommit ? `合并提交：${publicDelivery.mergeCommit}` : "合并提交：未记录",
  ];
}

function deliveryReleaseNoteItems(
  task: V1Issue | null,
  artifact: DeliveryArtifactState | null,
  projection?: TaskProjection | null,
) {
  if (!task) {
    return ["先选择一个任务。"];
  }

  const publicDelivery = projection?.publicDelivery ?? null;
  const publicRecordPath = publicDelivery?.changelogPath ?? publicDelivery?.releaseNotesUrl ?? null;
  if (publicRecordPath) {
    const changelogPath = publicDelivery?.changelogPath ?? null;
    const releaseNotesUrl = publicDelivery?.releaseNotesUrl ?? null;
    const prUrl = publicDelivery?.prUrl ?? null;
    return [
      changelogPath ? `CHANGELOG：${changelogPath}` : `Release notes：${releaseNotesUrl}`,
      prUrl ? `PR/MR：${prUrl}` : "PR/MR：未记录",
    ];
  }

  if (!artifact?.releaseNote) {
    return task.displayStatus === "in_review" || task.displayStatus === "done"
      ? ["公开交付缺失：当前状态应该已经有 CHANGELOG 或 release notes 记录。"]
      : ["当前阶段还没有交付说明。"];
  }

  const summaryLines = artifact.releaseNote.summaryLines.length
    ? artifact.releaseNote.summaryLines
    : ["交付说明没有更多摘要内容。"];

  return [
    artifact.releaseNote.title ? `标题：${artifact.releaseNote.title}` : "标题：未记录",
    ...summaryLines.map((line, index) => `摘要 ${index + 1}：${line}`),
  ];
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
    group.issues.find((issue) => issue.displayStatus === "in_progress") ??
    group.issues.find((issue) => issue.displayStatus === "todo") ??
    group.issues.find((issue) => issue.displayStatus === "in_review") ??
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

  const todoCount = group.issues.filter((issue) => issue.displayStatus === "todo").length;
  const reviewCount = group.issues.filter((issue) => issue.displayStatus === "in_review").length;
  const backlogCount = group.issues.filter((issue) => issue.displayStatus === "backlog").length;

  return [
    `总任务：${group.counts.issueCount}`,
    `正在做：${group.counts.activeIssueCount}`,
    `准备开工：${todoCount}`,
    `正在评审：${reviewCount}`,
    `待处理：${backlogCount}`,
    `已完成：${group.counts.doneIssueCount}`,
    `审计任务：${group.counts.auditIssueCount}`,
  ];
}

function projectPrioritySummaryItems(group: TaskProjectGroup) {
  if (!group.issues.length) {
    return ["还没有任务优先级记录。"];
  }

  const priority = groupHighestPriority(group.issues);
  const elevatedIssues = group.issues.filter((issue) => priorityRank(issue.priority) <= 1);

  return [
    `最高优先级：${displayPriority(priority)}`,
    ...(elevatedIssues.length
      ? elevatedIssues.slice(0, 5).map((issue) => `${issue.id}：${displayPriority(issue.priority)} · ${issue.title}`)
      : ["没有 P0/P1 任务。"]),
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

function projectCurrentLaneItems(group: TaskProjectGroup, projection?: ProjectProjection | null) {
  const items = projection?.lanes.current.length
    ? projection.lanes.current
        .map((issueId) => group.issues.find((issue) => issue.id === issueId))
        .filter((issue): issue is TaskIssueNode => Boolean(issue))
    : group.issues.filter((issue) => ["todo", "in_progress", "in_review", "blocked"].includes(issue.displayStatus));
  if (!items.length) {
    return ["当前没有正在推进的任务。"];
  }
  return items.slice(0, 4).map((issue) => `${issue.id} · ${displayStatusLabelZh(issue.displayStatus)} · ${issue.title}`);
}

function projectPastLaneItems(group: TaskProjectGroup, projection?: ProjectProjection | null) {
  const items = projection?.lanes.past.length
    ? projection.lanes.past
        .map((issueId) => group.issues.find((issue) => issue.id === issueId))
        .filter((issue): issue is TaskIssueNode => Boolean(issue))
    : group.issues.filter((issue) => ["done", "cancel"].includes(issue.displayStatus));
  if (!items.length) {
    return ["还没有已结束任务。"];
  }
  return items.slice(0, 4).map((issue) => `${issue.id} · ${displayStatusLabelZh(issue.displayStatus)} · ${issue.title}`);
}

function projectFutureLaneItems(group: TaskProjectGroup, projection?: ProjectProjection | null) {
  const items = projection?.lanes.future.length
    ? projection.lanes.future
        .map((issueId) => group.issues.find((issue) => issue.id === issueId))
        .filter((issue): issue is TaskIssueNode => Boolean(issue))
    : group.issues.filter((issue) => issue.displayStatus === "backlog");
  if (!items.length) {
    return ["后续没有待进入的任务。"];
  }
  return items.slice(0, 4).map((issue) => `${issue.id} · ${displayPriority(issue.priority)} · ${issue.title}`);
}

function projectNextScheduledIssue(
  group: TaskProjectGroup,
  currentIssueId: string | null,
  futureLaneIssueIds?: string[] | null,
) {
  const runnableIssues = group.issues.filter((issue) => !["done", "cancel"].includes(issue.displayStatus));
  if (!runnableIssues.length) {
    return null;
  }
  if (futureLaneIssueIds?.length) {
    const projectedNext = futureLaneIssueIds
      .map((issueId) => runnableIssues.find((issue) => issue.id === issueId))
      .find((issue): issue is TaskIssueNode => Boolean(issue));
    if (projectedNext) {
      return projectedNext;
    }
  }
  if (!currentIssueId) {
    return runnableIssues[0] ?? null;
  }
  const currentIndex = runnableIssues.findIndex((issue) => issue.id === currentIssueId);
  if (currentIndex >= 0) {
    return runnableIssues.slice(currentIndex + 1).find((issue) => issue.displayStatus === "backlog") ?? null;
  }
  return runnableIssues.find((issue) => issue.displayStatus === "backlog") ?? null;
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
    ...group.missingIssueIds.map((issueId) => `缺失任务引用：${issueId}`),
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
  return Boolean(outputs?.evidencePath?.trim());
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

function normalizePriority(priority?: string | null) {
  const normalized = (priority ?? "p2").toLowerCase();
  return normalized === "p0" || normalized === "p1" || normalized === "p2" || normalized === "p3"
    ? normalized
    : "p2";
}

function priorityRank(priority?: string | null) {
  return { p0: 0, p1: 1, p2: 2, p3: 3 }[normalizePriority(priority)];
}

function displayPriority(priority?: string | null) {
  return normalizePriority(priority).toUpperCase();
}

function groupHighestPriority(issues: TaskIssueNode[]) {
  return issues.reduce(
    (highest, issue) =>
      priorityRank(issue.priority) < priorityRank(highest)
        ? normalizePriority(issue.priority)
        : highest,
    "p3",
  );
}

function priorityStatusDotClass(priority?: string | null) {
  return `v16-priority-dot-${normalizePriority(priority)}`;
}

function priorityTextClass(priority?: string | null) {
  return `v16-priority-text-${normalizePriority(priority)}`;
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
  outputBundle: OutputBundleState,
  initializationStatus: ProjectInitializationStatus | null,
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
  const auditItems =
    sortAuditsByLatest(outputBundle.auditIndex?.audits ?? []).slice(0, 2).map((audit) => ({
      detail: `${audit.auditId} · ${artifactStatusLabel(audit.status)}`,
      id: `audit-${audit.auditId}`,
      target: "audit" as const,
      title: "审计页面同步结构",
    })) ?? [];

  const items = [...initializationItems, ...auditItems];
  if (items.length) {
    return items.slice(0, 4);
  }

  return [
    {
      detail: `${outputBundle.auditIndex?.audits.length ?? 0} 个审计`,
      id: "activity-task-flow",
      target: "tasks" as const,
      title: "任务状态流等待事件",
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
  issueCount: number,
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

  if (issueCount === 0) {
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

function isChineseAgentLocale(agentLocale?: string | null) {
  return (agentLocale ?? "").trim().toLowerCase().startsWith("zh");
}

function buildAgentPullRequestTemplate(task: V1Issue, agentLocale?: string | null) {
  return isChineseAgentLocale(agentLocale)
    ? buildAgentPullRequestTemplateZh(task)
    : buildAgentPullRequestTemplateEn(task);
}

function buildAgentPullRequestTemplateEn(task: V1Issue) {
  const issuePath = task.issuePath ?? `.agentflow/spec/issues/${task.id}.json`;
  const sourceSpec = task.sourceSpecPath ?? task.sourceSpecId ?? "Not recorded";
  const validationCommands = task.validationCommands.length
    ? task.validationCommands.map((command) => `- ${command}`)
    : ["- Not recorded"];
  const importantFiles = task.allowedFiles.length ? task.allowedFiles.map((path) => `- ${path}`) : ["- Not recorded"];
  const outputs = taskOutputItems(task).map((item) => `- ${item}`);

  return [
    "# AgentFlow Build Agent Pull Request",
    "",
    "AgentFlow keeps every Build Agent PR bound to one spec issue, one branch, one review gate, and one Done writeback.",
    "",
    "## Task",
    "",
    `- Issue ID: ${task.id}`,
    `- Source issue file: ${issuePath}`,
    `- Source SPEC: ${sourceSpec}`,
    "- Owner role: build-agent",
    "- Review role: human-reviewer",
    `- Priority: ${displayPriority(task.priority)}`,
    "- Branch: ",
    "- Merge mode: auto-merge-if-eligible / manual-merge fallback",
    "- Lease file: ",
    "",
    "## Summary",
    "",
    "<!-- What changed, why it changed, and what behavior is affected. -->",
    "",
    "## Plain-Language Summary",
    "",
    "<!-- Explain plainly: what changed, what did not change, validation results, behavior impact, and whether auto-merge is allowed. -->",
    "",
    "## Files Changed",
    "",
    ...importantFiles,
    "",
    "## Scope Checklist",
    "",
    "- [ ] This PR covers one AgentFlow issue only.",
    "- [ ] Touched paths match the issue allowed paths.",
    "- [ ] No unrelated refactors or formatting churn.",
    "- [ ] No forbidden paths were modified.",
    "- [ ] No audit report, findings, evidence map, or traceability files were written by Build Agent.",
    "- [ ] Spec facts and public requirement records were not modified unless explicitly allowed by the issue.",
    "- [ ] Public behavior is unchanged, or migration notes are included.",
    "",
    "## Build Agent Loop",
    "",
    "- [ ] Issue preflight completed and the issue moved from backlog to todo.",
    "- [ ] Current run was created by the official AgentFlow runtime entrypoint before source edits started.",
    "- [ ] Test design completed, with failing test evidence or a reason TDD does not apply.",
    "- [ ] Issue implementation stayed inside scope.",
    "- [ ] Sandbox verification completed.",
    "- [ ] PR/MR created with validation evidence.",
    "- [ ] Merge mode is respected.",
    "- [ ] Done writeback is not performed before PR/MR merge.",
    "",
    "## Evidence",
    "",
    "- Evidence file:",
    "- Release delivery:",
    "- Plain-language summary:",
    "- Commands run:",
    ...validationCommands,
    "- Command result summary:",
    "- Tests added or updated:",
    "- Tests not run and reason:",
    "- Browser or screenshot evidence:",
    "",
    "## Expected Outputs",
    "",
    ...outputs,
    "",
    "## Impact",
    "",
    "- Runtime behavior impact:",
    "- Public API impact:",
    "- Data or file format impact:",
    "- Migration note status:",
    "- Release delivery impact:",
    "",
    "## Rollback Plan",
    "",
    "<!-- Exact revert or rollback steps. -->",
    "",
    "## Review Gate",
    "",
    "- [ ] Owner role did not approve its own task.",
    "- [ ] Verification evidence is present or explicitly not required for this execution risk.",
    "- [ ] `in_review` is not treated as `done`.",
    "- [ ] `BLOCKED` is not treated as `DONE`.",
    "- [ ] `QA_PASSED` is not treated as `DONE`.",
  ].join("\n");
}

function buildAgentPullRequestTemplateZh(task: V1Issue) {
  const issuePath = task.issuePath ?? `.agentflow/spec/issues/${task.id}.json`;
  const sourceSpec = task.sourceSpecPath ?? task.sourceSpecId ?? "未记录";
  const validationCommands = task.validationCommands.length
    ? task.validationCommands.map((command) => `- ${command}`)
    : ["- 未记录"];
  const importantFiles = task.allowedFiles.length ? task.allowedFiles.map((path) => `- ${path}`) : ["- 未记录"];
  const outputs = taskOutputItems(task).map((item) => `- ${item}`);

  return [
    "# AgentFlow Build Agent PR",
    "",
    "AgentFlow 要求每个 Build Agent PR 只对应一个 spec issue、一个分支、一个 review gate 和一次 Done 写回。",
    "",
    "## 任务",
    "",
    `- 任务 ID：${task.id}`,
    `- 任务文件：${issuePath}`,
    `- 来源 SPEC：${sourceSpec}`,
    "- 执行角色：build-agent",
    "- 审查角色：human-reviewer",
    `- 优先级：${displayPriority(task.priority)}`,
    "- 分支：",
    "- 合并模式：auto-merge-if-eligible / manual-merge fallback",
    "- Lease 文件：",
    "",
    "## 变更摘要",
    "",
    "<!-- 说明改了什么、为什么改、影响哪些行为。 -->",
    "",
    "## 大白话说明",
    "",
    "<!-- 直接说明：改了什么、没改什么、验证结果、行为影响，以及是否允许自动合并。 -->",
    "",
    "## 变更文件",
    "",
    ...importantFiles,
    "",
    "## 范围检查",
    "",
    "- [ ] 这个 PR 只处理一个 AgentFlow issue。",
    "- [ ] 修改路径符合 issue 允许路径。",
    "- [ ] 没有无关重构或格式化噪音。",
    "- [ ] 没有修改禁止路径。",
    "- [ ] Build Agent 没有写审计报告、findings、evidence map 或 traceability 文件。",
    "- [ ] 没有修改 input facts 和 Approved SPEC，除非 issue 明确允许。",
    "- [ ] 公共行为未改变，或已补充迁移说明。",
    "",
    "## Build Agent Loop",
    "",
    "- [ ] 已完成执行前置检测，issue 已从 backlog 进入 todo。",
    "- [ ] 已完成测试设计，包含失败测试证据，或说明 TDD 不适用的原因。",
    "- [ ] 实现过程没有越过 issue 范围。",
    "- [ ] 已完成沙箱验证。",
    "- [ ] 已创建 PR/MR，并附带验证证据。",
    "- [ ] 已遵守 mergeMode。",
    "- [ ] PR/MR 合并前没有写回 Done。",
    "",
    "## 证据",
    "",
    "- Evidence 文件：",
    "- Release delivery：",
    "- 大白话说明：",
    "- 已运行命令：",
    ...validationCommands,
    "- 命令结果摘要：",
    "- 新增或更新的测试：",
    "- 未运行的测试及原因：",
    "- 浏览器或截图证据：",
    "",
    "## 预期输出",
    "",
    ...outputs,
    "",
    "## 影响",
    "",
    "- 运行时行为影响：",
    "- 公共 API 影响：",
    "- 数据或文件格式影响：",
    "- 迁移说明状态：",
    "- Release delivery 影响：",
    "",
    "## 回滚计划",
    "",
    "<!-- 写清楚精确的 revert 或回滚步骤。 -->",
    "",
    "## Review Gate",
    "",
    "- [ ] Owner role 没有审批自己的任务。",
    "- [ ] 验证证据已提供，或该风险级别明确不需要。",
    "- [ ] `in_review` 没有被当成 `done`。",
    "- [ ] `BLOCKED` 没有被当成 `DONE`。",
    "- [ ] `QA_PASSED` 没有被当成 `DONE`。",
  ].join("\n");
}

function buildCodexHandoff(task: V1Issue, agentLocale?: string | null) {
  const validationCommandTemplates = task.validationCommands.slice(0, 3).map((command) => ({
    args: command.split(" ").slice(1),
    exitCode: 0,
    label: command,
    program: command.split(" ")[0] ?? command,
    source: "build-agent",
  }));
  const changedFileTemplates = task.allowedFiles.slice(0, 3).map((path) => ({
    changeType: "modified",
    deletions: 0,
    insertions: 0,
    path,
  }));
  const handoffPackage =
    task.issueCategory === "audit"
      ? {
          agentInstruction: agentInstructionForTask(task),
          auditId: task.auditId,
          auditOutputDir: task.auditOutputDir,
          codexThreadName: codexThreadNameForRole(task.requiredAgentRole),
          expectedOutputs: task.expectedOutputs,
          handoffId: task.handoffId,
          handoffVersion: "agent-handoff.v1",
          issueCategory: "audit",
          issueId: task.id,
          priority: task.priority,
          projectId: task.projectId,
          requiredAgentRole: task.requiredAgentRole ?? "audit-agent",
          sourceDeliveryPath: task.sourceDeliveryPath,
          sourceReleaseId: task.sourceReleaseId,
        }
      : {
          agentInstruction: agentInstructionForTask(task),
          codexThreadName: codexThreadNameForRole(task.requiredAgentRole),
          runtimeStart: {
            cli: "target/release/agentflow build-agent start --issue-id <issue-id> after cargo build --release --bin agentflow, or target/debug/agentflow build-agent start --issue-id <issue-id>",
            issueId: task.id,
          },
          reviewPreparation: {
            cli: "target/release/agentflow build-agent prepare-review --request <completion-request.json> after cargo build --release --bin agentflow, or target/debug/agentflow build-agent prepare-review --request <completion-request.json>",
            request: {
              changedFiles: changedFileTemplates,
              issueId: task.id,
              runId: "<run-id-from-build-agent-start>",
              validationCommands: validationCommandTemplates,
            },
          },
          mergeProofWriteback: {
            cli: "target/release/agentflow build-agent write-merge-proof --issue-id <issue-id> --run-id <run-id> --provider <github|gitlab> --merge-mode <auto-merge-if-eligible|manual-merge> [--remote-url <url>] [--merged] after cargo build --release --bin agentflow, or target/debug/agentflow build-agent write-merge-proof --issue-id <issue-id> --run-id <run-id> --provider <github|gitlab> --merge-mode <auto-merge-if-eligible|manual-merge> [--remote-url <url>] [--merged]",
            request: {
              issueId: task.id,
              merged: false,
              mergeMode: "<auto-merge-if-eligible|manual-merge>",
              provider: "<github|gitlab>",
              remoteUrl: "<pr-or-mr-url>",
              runId: "<run-id-from-build-agent-start>",
            },
          },
          completionWriteback: {
            cli: "target/release/agentflow build-agent complete --request <completion-request.json> after cargo build --release --bin agentflow, or target/debug/agentflow build-agent complete --request <completion-request.json>",
            request: {
              changedFiles: changedFileTemplates,
              issueId: task.id,
              runId: "<run-id-from-build-agent-start>",
              validationCommands: validationCommandTemplates,
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
          priority: task.priority,
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
    `Codex 线程：${codexThreadNameForRole(task.requiredAgentRole)}`,
    `优先级：${displayPriority(task.priority)}`,
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
    "- AgentFlow 当前 spec issue 是唯一任务源。",
    "- handoff package 只是当前 issue 的派生快照。",
    "- executionPipeline 只是当前 issue 合同的一部分，不是独立任务源。",
    "- 不要把外部 issue、任务、计划、队列、线程或工具状态当成任务源。",
    "- 不要用外部状态拆分、重排或推进 AgentFlow 任务。",
    "- GitHub/GitLab 命令只允许用于当前 executionPipeline 里的 PR/MR 阶段。",
    "- 不要越过任务边界。",
    "- 不要手写 `.agentflow/tasks/**` 或 `.agentflow/events/**`。",
    "- 不要把“不要手写 `.agentflow/**`”理解成不能调用 AgentFlow 官方 loop 命令；官方 run 创建、Context Pack 生成和 complete 写回都必须走 AgentFlow 入口。",
    ...(task.issueCategory === "spec"
      ? [
          "",
          "## Build Agent 执行流程",
          ...taskExecutionPipelineItems(task).map((item) => `- ${item}`),
          "",
          "## 合并规则",
          "- 创建 PR/MR 前必须完成执行前置检测、测试设计和沙箱验证。",
          "- 创建 PR/MR 不是终点。Draft PR/MR 只是中间产物，不能直接写回 Done。",
          "- mergeMode = auto-merge-if-eligible：GitHub 执行 `gh pr ready` 和 `gh pr merge --auto`；GitLab 执行 `glab mr update --ready` 和 `glab mr merge --auto-merge`；轮询 PR/MR merged 状态。",
          "- 自动合并条件不满足时，回落到 manual-merge：把 PR/MR 标记 ready 后 issue 保持 in_review，等待人合并；本地检测确认 PR/MR merged 后继续写回。",
          "- PR/MR 合并后才写回 Done。",
          "- 进入 in_progress 前必须先执行 `agentflow build-agent start --issue-id <issue-id>`；没有 run 不得开始改源码。",
          "- 进入 in_progress 前必须确认 Context Pack 可读，且当前工作区没有未提交的用户源码改动。",
          "- 本地验证完成后执行 `agentflow build-agent prepare-review --request <completion-request.json>`，把 issue 推到 in_review。",
          "- PR/MR ready 或 merged 后执行 `agentflow build-agent write-merge-proof ...`，把 remoteUrl / mergeMode / merged 状态写回官方 review 证明。",
          "- 写回 Done 前必须确认当前 AgentFlow CLI 支持 `build-agent start`、`build-agent prepare-review`、`build-agent write-merge-proof` 和 `build-agent complete`。",
          "- 如果使用 `target/release/agentflow`，必须先运行 `cargo build --release --bin agentflow`；否则使用 `target/debug/agentflow`。",
          "- 不要直接复用可能过期的 `target/release/agentflow`。",
          "",
          "## PR 描述模板",
          "```md",
          buildAgentPullRequestTemplate(task, agentLocale),
          "```",
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
          "- 进入执行前先运行 `target/release/agentflow build-agent start --issue-id <issue-id>`；如果 release binary 过期或不支持，使用 `target/debug/agentflow build-agent start --issue-id <issue-id>`。",
          "- 记录 `build-agent start` 返回的 `runId`，后续 `prepare-review`、`write-merge-proof`、`complete` 都必须复用这个 `runId`。",
          "- 本地验证完成后运行 `target/release/agentflow build-agent prepare-review --request <completion-request.json>`；如果 release binary 过期或不支持，使用 `target/debug/agentflow build-agent prepare-review --request <completion-request.json>`。",
          "- PR/MR ready 或 merged 后运行 `target/release/agentflow build-agent write-merge-proof --issue-id <issue-id> --run-id <run-id> --provider <github|gitlab> --merge-mode <auto-merge-if-eligible|manual-merge> [--remote-url <url>] [--merged]`；如果 release binary 过期或不支持，使用对应 debug binary。",
          "- 完成代码任务并确认 PR 已合并后，调用已验证的新 CLI 写回。",
          "- 使用 `target/release/agentflow` 前必须先运行 `cargo build --release --bin agentflow`。",
          "- 如果 release binary 不支持 `build-agent complete`，使用 `target/debug/agentflow build-agent complete --request <completion-request.json>`。",
          "- request.issueId 必须等于当前任务 id。",
          "- request.runId 必须等于 `build-agent start` 返回的 `runId`。",
          "- request.changedFiles 填写实际修改文件。",
          "- request.validationCommands 填写已执行的验证命令和 exitCode。",
          "- AgentFlow 会自动生成规范 run、evidence、task events，并把任务派生成已完成；公开交付记录写入 PR/MR body、CHANGELOG 或 release notes。",
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
      codexThreadName: guide.threadName,
      agentThreadName: guide.threadName,
      summary: guide.summary,
      cannotDo: guide.cannotDo,
    })),
    matrix: [
      {
        agentRole: "spec-agent",
        handlesIssueCategory: [],
        writes: ["docs/requirements/**", ".agentflow/spec/**"],
      },
      {
        agentRole: "build-agent",
        handlesIssueCategory: ["spec"],
        writes: [".agentflow/tasks/<issue-id>/runs/**", ".agentflow/tasks/<issue-id>/evidence/**", "PR/MR body", "CHANGELOG.md or release notes"],
      },
      {
        agentRole: "audit-agent",
        handlesIssueCategory: ["audit"],
        writes: [".agentflow/audit/**"],
      },
    ],
  };
}

function buildNextActionLabel(action: string) {
  const labels: Record<string, string> = {
    "confirm-goal-draft-preview": "确认 Goal 草稿预览",
    "confirm-plan-draft-preview": "确认 Plan 草稿预览",
    "confirm-project-brain": "确认 Project Brain",
    "create-goal-draft-preview": "生成 Goal 草稿预览",
    "create-plan-draft-preview": "生成 Plan 草稿预览",
    "enter-completion-decision": "进入完成判断",
    "execute-issue": "执行任务",
    "materialize-spec-project-and-issues": "物化 SpecProject / SpecIssue",
    "prepare-public-delivery": "生成公开交付记录",
    "resolve-project-brain-blocker": "处理 Project Brain 阻断",
    "run-goal-recheck": "重新检查项目目标",
    "start-project-loop": "进入项目循环",
    "start-new-requirement": "开始新需求",
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
    complete: "已完成",
    delivered: "已交付",
    drafted: "已生成草稿",
    done: "已完成",
    failed: "失败",
    missing: "缺失",
    pass: "通过",
    passed: "通过",
    "passed-with-warnings": "通过，有警告",
    pending: "待处理",
    published: "已发布",
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

function mcpProviderLabel(provider?: string | null) {
  const labels: Record<string, string> = {
    codex: "Codex",
    github: "GitHub",
    gitlab: "GitLab",
  };
  return labels[(provider ?? "").toLowerCase()] ?? (provider || "未记录");
}

function mcpSessionStatusLabelZh(status?: string | null) {
  const labels: Record<string, string> = {
    claimed: "已接单",
    cancelled: "已取消",
    done: "已完成",
    failed: "失败",
    "in-review": "正在评审",
    interrupted: "已中断",
    queued: "等待启动",
    requested: "已发起启动请求",
    running: "正在做",
    starting: "启动中",
  };
  return status ? labels[status] ?? status : "未记录";
}

function mcpSessionStatusTone(status?: string | null): StatusChipStatus {
  const tones: Record<string, StatusChipStatus> = {
    claimed: "ready",
    cancelled: "blocked",
    done: "done",
    failed: "failed",
    "in-review": "warning",
    interrupted: "warning",
    queued: "idle",
    requested: "ready",
    running: "working",
    starting: "ready",
  };
  return status ? tones[status] ?? "idle" : "idle";
}

function mcpLaunchModeLabelZh(mode?: McpSessionSnapshot["launchMode"] | null) {
  const labels: Record<McpSessionSnapshot["launchMode"], string> = {
    "app-server-thread": "App 服务线程",
    "cli-exec-prompt-file": "CLI Prompt 文件",
    "cli-exec-stdin": "CLI 标准输入",
    "mcp-remote-session": "远程 MCP 会话",
  };
  return mode ? labels[mode] ?? mode : "未记录";
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
    publicDelivery: "公开交付",
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
    files: "文件",
    home: "工作台",
    tasks: "任务流转",
  };
  return labels[page] ?? "工作台";
}

function executeWorkspaceStatusLabel(status: string | undefined, source: ExecuteStatusState["source"], error: string | null) {
  if (error) {
    return "异常";
  }

  if (source === "loading") {
    return "检查中";
  }

  const labels: Record<string, string> = {
    blocked: "已阻断",
    failed: "异常",
    missing: "未初始化",
    ready: "已就绪",
  };
  return labels[status ?? ""] ?? "未检查";
}

function executeWorkspaceStatusTone(status: string | undefined, error: string | null): StatusChipStatus {
  if (error) {
    return "failed";
  }

  const tones: Record<string, StatusChipStatus> = {
    blocked: "blocked",
    failed: "failed",
    missing: "idle",
    ready: "ready",
  };
  return tones[status ?? ""] ?? "idle";
}

function taskMcpSessionItems(session: McpSessionSnapshot | null, mcpSessionsState: McpSessionsState) {
  if (session) {
    return [
      `状态：${mcpSessionStatusLabelZh(session.status)}`,
      `Run：${session.runId}`,
      `分支：${session.branchName ?? "未记录"}`,
      `任务包：${session.launchRequestPath}`,
      `计划文件：${session.planPath}`,
      `PR/MR：${session.prUrl ?? "未记录"}`,
    ];
  }
  if (mcpSessionsState.source === "loading") {
    return ["正在读取执行会话。"];
  }
  if (mcpSessionsState.error) {
    return [`读取失败：${mcpSessionsState.error}`];
  }
  return ["还没有执行会话。"];
}

function executeSessionItems(session: McpSessionSnapshot) {
  return [
    `会话 ID：${session.sessionId}`,
    `任务：${session.issueId}`,
    `Run：${session.runId}`,
    `启动模式：${mcpLaunchModeLabelZh(session.launchMode)}`,
    `分支：${session.branchName ?? "未记录"}`,
    `任务包：${session.launchRequestPath}`,
    `计划文件：${session.planPath}`,
    `日志：${session.logPath ?? "未记录"}`,
    `PR/MR：${session.prUrl ?? "未记录"}`,
    `合并状态：${session.mergeState ?? "未记录"}`,
  ];
}

function workflowStageText(stage?: string | null) {
  const labels: Record<string, string> = {
    "audit-completed": "审计完成",
    "audit-requested": "审计已请求",
    "completion-ready": "等待完成判断",
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

  if (selectedTask?.displayStatus === "todo") {
    return "waiting-for-agent";
  }
  if (selectedTask?.displayStatus === "in_progress") {
    return "agent-running";
  }
  if (selectedTask?.displayStatus === "in_review") {
    return "ready-for-audit";
  }
  if (selectedTask?.displayStatus === "done") {
    return "delivered";
  }

  const labels: Record<string, string> = {
    "audit-completed": "audit-completed",
    "audit-requested": "audit-requested",
    "completion-ready": "needs-completion-decision",
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
    initialization: "展示项目初始化摘要。这里不重跑初始化，不写旧示例数据。",
    panel: "展示项目现场读取结果和上下文包摘要。",
    settings: "展示本地设置、文件阅读器和工作台数据源状态。",
    spec: "展示公开需求拆出的本地任务合同。普通页面只展示人能读懂的摘要。",
    state: "展示全局派生状态、门禁、阻断和下一步动作。",
    tasks: "展示任务状态流和项目任务集合投影。",
  };
  return summaries[categoryId] ?? "这里展示开发者调试信息。普通页面不显示原始 JSON。";
}

function advancedFilesForCategory(
  categoryId: string,
  categoryValue?: unknown,
  stateJsonFilesState?: AdvancedStateJsonFilesState,
): AdvancedJsonFile[] {
  const files: Record<string, AdvancedJsonFile[]> = {
    agentRoles: [
      {
        name: ".agentflow/define/agent/roles.json",
        displayName: "roles.json",
        description: "三类 Agent 的可处理任务和写入边界",
      },
      { name: "AGENTS.md", displayName: "AGENTS.md", description: "根级 Agent 入口规则" },
      {
        name: ".agentflow/define/agent/Agentflow.md",
        displayName: "Agentflow.md",
        description: "AgentFlow 工作手册",
      },
    ],
    audit: [
      { name: "index.json", description: "审计报告索引" },
      { name: "audit.json", description: "审计结论和检查结果" },
      { name: "evidence-map.json", description: "证据链映射" },
      { name: "traceability.json", description: "规格、任务和交付追溯" },
    ],
    initialization: [
      { name: "base-release-initialization.json", description: "基础发布初始化摘要" },
      { name: "recent-project-context.json", description: "现有项目最近提交上下文" },
      { name: "git-context.json", description: "本地 Git 上下文索引" },
    ],
    panel: [
      { name: "manifest.json", description: "项目现场摘要" },
      { name: "context-packs/*.json", description: "上下文包" },
      { name: "diagnostics.json", description: "诊断快照" },
    ],
    spec: [
      { name: "index.json", description: "规格、项目和任务索引" },
      { name: "issues/*.json", description: "任务合同来源" },
      { name: "projects/*.json", description: "项目任务集合" },
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
    tasks: [
      { name: "issue-status.json", description: "任务状态索引" },
      { name: "projections/tasks/*.json", description: "单任务状态流投影" },
      { name: "projections/projects/*.json", description: "项目任务集合投影" },
    ],
  };
  const categoryFiles = files[categoryId] ?? files.state;
  if (categoryId !== "state") {
    return categoryFiles.map((file) => ({ ...file, value: categoryValue }));
  }
  return categoryFiles.map((file) => ({
    ...file,
    error: stateJsonFilesState?.errors[file.name] ?? null,
    loading: stateJsonFilesState?.source === "loading" && stateJsonFilesState.files[file.name] === undefined,
    value: stateJsonFilesState?.files[file.name],
  }));
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
