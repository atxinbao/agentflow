import { invoke } from "@tauri-apps/api/core";
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
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useState, type ReactNode } from "react";
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
  ListRow,
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
import { DesignSystemPreview } from "./features/design-system";
import { useAgentManual } from "./features/agent-manual";
import { useExecuteStatus } from "./features/execute";
import { useInputSnapshot, useInputStatus } from "./features/input";
import { OutputAuditPanel, useOutputStatus, type OutputStatusState } from "./features/output";
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
import { AgentStatusBar, buildAgentStatusItems } from "./features/status-channel";
import {
  buildAppInteractionState,
  buildAuditInteractionState,
  buildDeliveryInteractionState,
  buildTaskInteractionState,
  displayStatusLabelZh,
  pickTaskId,
  taskActionLabel,
  taskActionsForStatus,
  type AppInteractionState,
  type ButtonInteractionState,
  type TaskInteractionAction,
} from "./interaction/viewModels";
import type {
  AgentStatusChannelItem,
  AuditIndex,
  AuditIndexEntry,
  HumanAuditReport,
  InputIssue,
  IssueDisplayStatus,
  IssueStatusIndex,
  IssueContract,
  OutputIndex,
  OutputIndexEntry,
  ProjectMilestoneIssueViewModelSnapshot,
  V1Issue,
  WorkbenchSnapshot,
} from "./types";
import "./AppShell.css";

type Provider = "ChatGPT" | "Claude" | "DeepSeek";
type AppPage = "home" | "tasks" | "files" | "delivery" | "audit" | "advanced";
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

const onboardingSteps = ["选择项目", "环境准备", "认识智能体", "确认意图", "完成引导"] as const;

const interactionStorageKeys = {
  activePage: "agentflow.interaction.activePage.v1",
  handedOffIssues: "agentflow.interaction.handedOffIssues.v1",
  onboardingComplete: "agentflow.interaction.onboardingComplete.v1",
  projectRoot: "agentflow.interaction.projectRoot.v1",
  provider: "agentflow.interaction.provider.v1",
} as const;

function readStoredProvider(): Provider | null {
  const value = window.localStorage.getItem(interactionStorageKeys.provider);
  return value === "ChatGPT" || value === "Claude" || value === "DeepSeek" ? value : null;
}

function readStoredPage(): AppPage {
  const value = window.localStorage.getItem(interactionStorageKeys.activePage);
  return pages.some((page) => page.id === value) ? (value as AppPage) : "home";
}

function readStoredProjectRoot() {
  return window.localStorage.getItem(interactionStorageKeys.projectRoot);
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

function App() {
  const [connectedProvider, setConnectedProvider] = useState<Provider | null>(() =>
    isBrowserPreviewRuntime() ? "ChatGPT" : readStoredProvider(),
  );
  const [onboardingComplete, setOnboardingComplete] = useState(() =>
    isBrowserPreviewRuntime() ? true : readStoredBoolean(interactionStorageKeys.onboardingComplete),
  );
  const [firstRunOpen, setFirstRunOpen] = useState(() => Boolean(readStoredProvider()) && !onboardingComplete);
  const [projectRoot, setProjectRoot] = useState<string | null>(
    isBrowserPreviewRuntime() ? BROWSER_PREVIEW_PROJECT_ROOT : readStoredProjectRoot(),
  );
  const [activePage, setActivePage] = useState<AppPage>(() => (isBrowserPreviewRuntime() ? "home" : readStoredPage()));
  const [taskSearch, setTaskSearch] = useState("");
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [selectedDeliveryRunId, setSelectedDeliveryRunId] = useState<string | null>(null);
  const [selectedAuditId, setSelectedAuditId] = useState<string | null>(null);
  const [outputRefreshToken, setOutputRefreshToken] = useState(0);
  const [selectedIntent, setSelectedIntent] = useState("我要新增功能");
  const [onboardingFeedback, setOnboardingFeedback] = useState<string | null>(null);
  const [taskActionFeedback, setTaskActionFeedback] = useState<string | null>(null);
  const [taskCopyState, setTaskCopyState] = useState<ButtonInteractionState>("enabled");
  const [handedOffIssues, setHandedOffIssues] = useState<Set<string>>(() => readStoredIssueSet());

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
  const inputStatusState = useInputStatus(projectRoot);
  const inputSnapshotState = useInputSnapshot(projectRoot);
  const executeStatusState = useExecuteStatus(projectRoot);
  const outputStatusState = useOutputStatus(projectRoot, outputRefreshToken);
  const stateStatusState = useStateStatus(projectRoot);
  const issueStatusIndexState = useIssueStatusIndex(projectRoot, outputRefreshToken);
  const workspaceData = useWorkspaceData(projectRoot);
  const outputBundle = useOutputBundle(projectRoot, outputRefreshToken);

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
    }
  }, [projectRoot]);

  useEffect(() => {
    window.localStorage.setItem(interactionStorageKeys.activePage, activePage);
  }, [activePage]);

  useEffect(() => {
    window.localStorage.setItem(interactionStorageKeys.handedOffIssues, JSON.stringify([...handedOffIssues]));
  }, [handedOffIssues]);

  useEffect(() => {
    if (!projectRoot) {
      return;
    }

    void loadProjectFiles(projectRoot);
    void loadAgentManual(projectRoot);
  }, [projectRoot]);

  useEffect(() => {
    if (!projectRoot) {
      return;
    }
    if (activePage === "files") {
      void loadProjectFiles(projectRoot);
    }
    if (activePage === "home" || activePage === "tasks") {
      void prepareProjectPanel(projectRoot);
    }
    if (activePage === "delivery" || activePage === "audit") {
      setOutputRefreshToken((current) => current + 1);
    }
  }, [activePage, projectRoot]);

  const agentStatusItems = useMemo(
    () =>
      buildAgentStatusItems({
        agentManualState,
        executeStatusState,
        inputStatusState,
        outputStatusState,
        projectFilesState,
        projectPanelState,
        stateStatusState,
      }),
    [
      agentManualState,
      executeStatusState,
      inputStatusState,
      outputStatusState,
      projectFilesState,
      projectPanelState,
      stateStatusState,
    ],
  );

  const tasks = useMemo(
    () =>
      buildTaskItems(
        inputSnapshotState.snapshot?.issues ?? [],
        issueStatusIndexState.index,
        workspaceData.projectViewModel,
        workspaceData.workbench,
      ),
    [inputSnapshotState.snapshot, issueStatusIndexState.index, workspaceData.projectViewModel, workspaceData.workbench],
  );
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
  const selectedTaskCandidateId = useMemo(
    () => pickTaskId(tasks, selectedTaskId, activeIssueId),
    [activeIssueId, selectedTaskId, tasks],
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
  const appInteractionState: AppInteractionState = useMemo(
    () =>
      buildAppInteractionState({
        activePage,
        hasError: Boolean(workspaceData.error || outputBundle.error),
        onboardingComplete,
        projectLoading: projectFilesState.loading || workspaceData.source === "loading",
        projectRoot,
        providerConnected: Boolean(connectedProvider),
        workspaceBlocked: Boolean(stateStatusState.status?.blockers.length),
      }),
    [
      activePage,
      connectedProvider,
      onboardingComplete,
      outputBundle.error,
      projectFilesState.loading,
      projectRoot,
      stateStatusState.status?.blockers.length,
      workspaceData.error,
      workspaceData.source,
    ],
  );

  useEffect(() => {
    const nextTaskId = pickTaskId(tasks, selectedTaskId, activeIssueId);
    if (nextTaskId !== selectedTaskId) {
      setSelectedTaskId(nextTaskId);
    }
  }, [activeIssueId, selectedTaskId, tasks]);

  async function chooseProjectFolder() {
    if (isBrowserPreviewRuntime()) {
      setProjectRoot(BROWSER_PREVIEW_PROJECT_ROOT);
      setOnboardingFeedback("浏览器预览使用 mock 项目现场，不会读取或写入真实本地项目。");
      return;
    }

    try {
      const selectedRoot = await invoke<string | null>("choose_existing_project_folder");
      const normalizedRoot = selectedRoot ? normalizeProjectRootKey(selectedRoot) : null;
      if (!normalizedRoot) {
        return;
      }

      setOnboardingFeedback("正在准备项目工作规则和现场。");
      await invoke("prepare_local_project_workspace", {
        appLocale: detectAppLocale(),
        projectRoot: normalizedRoot,
      });
      setProjectRoot(normalizedRoot);
      setOnboardingFeedback("项目已准备好。");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setOnboardingFeedback(message);
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
    setOutputRefreshToken((current) => current + 1);
  }

  async function handleTaskAction(action: TaskInteractionAction, task: V1Issue) {
    setTaskActionFeedback(null);
    if (action === "copy-handoff") {
      setTaskCopyState("loading");
      try {
        await navigator.clipboard.writeText(buildCodexHandoff(task));
        setTaskCopyState("success");
        setTaskActionFeedback("任务包已复制到剪贴板。请手动交给 Codex。");
        window.setTimeout(() => setTaskCopyState("enabled"), 1400);
      } catch {
        setTaskCopyState("error");
        setTaskActionFeedback("复制失败。请手动复制 Codex Handoff Package。");
      }
      return;
    }

    if (action === "mark-handed-off") {
      setHandedOffIssues((current) => new Set(current).add(task.id));
      setTaskActionFeedback("已做本地标记。AgentFlow 不会自动控制 Codex。");
      return;
    }

    if (action === "check-writeback") {
      refreshWorkspace();
      const delivery = findDeliveryForTask(outputBundle.outputIndex?.releaseDeliveries ?? [], task.id);
      if (delivery) {
        setSelectedDeliveryRunId(delivery.runId);
        setActivePage("delivery");
      } else {
        setTaskActionFeedback("还没有检测到 Codex 写回结果。");
      }
      return;
    }

    if (action === "view-delivery") {
      const delivery = findDeliveryForTask(outputBundle.outputIndex?.releaseDeliveries ?? [], task.id);
      if (delivery) {
        setSelectedDeliveryRunId(delivery.runId);
        setActivePage("delivery");
      } else {
        setTaskActionFeedback("还没有交付结果。Codex 写回后会显示在交付页。");
      }
      return;
    }

    if (action === "request-audit") {
      const delivery = findDeliveryForTask(outputBundle.outputIndex?.releaseDeliveries ?? [], task.id);
      if (delivery) {
        setSelectedDeliveryRunId(delivery.runId);
      }
      setActivePage("audit");
      return;
    }

    if (action === "view-audit") {
      const audit = outputBundle.auditIndex?.audits.at(-1) ?? null;
      if (audit) {
        setSelectedAuditId(audit.auditId);
        setActivePage("audit");
      } else {
        setTaskActionFeedback("还没有审计报告。交付完成后可以请求审计。");
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
    if (isBrowserPreviewRuntime()) {
      setProjectRoot(BROWSER_PREVIEW_PROJECT_ROOT);
    }
  }

  function completeOnboarding() {
    setOnboardingComplete(true);
    setFirstRunOpen(false);
    setActivePage("home");
    refreshWorkspace();
  }

  if (!connectedProvider) {
    return <LoginModal onConnect={handleLogin} />;
  }

  const projectDisplayName = projectNameFromPath(projectRoot ?? "") || "未选择项目";
  const toolbar = (
    <Toolbar
      activePage={activePage}
      onRefresh={refreshWorkspace}
      onSearchChange={setTaskSearch}
      taskSearch={taskSearch}
    />
  );

  return (
    <>
      <AppShell
        activePage={activePage}
        connectedProvider={connectedProvider}
        inspector={activePage === "home" ? <InspectorPanel nextStep={nextStep} selectedTask={selectedTask} /> : null}
        onPageChange={setActivePage}
        projectName={projectDisplayName}
        projectRoot={projectRoot}
        statusBar={
          <StatusBar
            agentStatusItems={agentStatusItems}
            connectedProvider={connectedProvider}
            projectName={projectDisplayName}
            projectRoot={projectRoot}
            appInteractionState={appInteractionState}
            stateStatus={stateStatusState.status}
          />
        }
        toolbar={toolbar}
      >
        {activePage === "home" ? (
          <ProjectHomePage
            nextStep={nextStep}
            onOpenAudit={() => setActivePage("audit")}
            onOpenDelivery={() => setActivePage("delivery")}
            onOpenFiles={() => setActivePage("files")}
            onOpenTasks={() => setActivePage("tasks")}
            onCheckWriteback={() => selectedTask && void handleTaskAction("check-writeback", selectedTask)}
            outputStatusState={outputStatusState}
            projectPanelState={projectPanelState}
            projectRoot={projectRoot}
            selectedTask={selectedTask}
            stateStatusState={stateStatusState}
          />
        ) : null}
        {activePage === "tasks" ? (
          <TasksPage
            actionFeedback={taskActionFeedback}
            actions={taskInteractionState.actions}
            copyState={taskCopyState}
            handedOff={selectedTask ? handedOffIssues.has(selectedTask.id) : false}
            onTaskAction={(action, task) => void handleTaskAction(action, task)}
            onSelectTask={setSelectedTaskId}
            selectedTask={selectedTask}
            tasks={filteredTasks}
          />
        ) : null}
        {activePage === "files" ? (
          <FilesPage
            fileState={projectFilesState}
            onChangeViewMode={setProjectFileViewMode}
            onLoadDirectoryPage={loadProjectDirectoryPage}
            onLoadTextRange={loadProjectFileTextRange}
            onSearchFiles={searchProjectFiles}
            onSelectFile={selectProjectFile}
          />
        ) : null}
        {activePage === "delivery" ? (
          <DeliveryPage
            onOpenAudit={() => setActivePage("audit")}
            onSelectDelivery={setSelectedDeliveryRunId}
            outputBundle={outputBundle}
            outputStatusState={outputStatusState}
            selectedDeliveryRunId={selectedDeliveryRunId}
            selectedTask={selectedTask}
          />
        ) : null}
        {activePage === "audit" ? (
          <AuditPage
            onAuditRequested={() => setOutputRefreshToken((current) => current + 1)}
            onSelectAudit={setSelectedAuditId}
            outputBundle={outputBundle}
            outputStatusState={outputStatusState}
            projectRoot={projectRoot}
            selectedAuditId={selectedAuditId}
          />
        ) : null}
        {activePage === "advanced" ? (
          <AdvancedPage
            agentManualState={agentManualState}
            executeStatusState={executeStatusState}
            inputStatusState={inputStatusState}
            outputBundle={outputBundle}
            outputStatusState={outputStatusState}
            projectFilesState={projectFilesState}
            projectPanelState={projectPanelState}
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
    setState((current) => ({ ...current, error: null, source: "loading" }));
    void Promise.all([
      invoke<OutputIndex>("load_output_index", { projectRoot }),
      invoke<AuditIndex>("load_audit_index", { projectRoot }),
    ])
      .then(async ([outputIndex, auditIndex]) => {
        const latestAudit = [...auditIndex.audits].sort((left, right) => left.requestedAt - right.requestedAt).at(-1);
        const auditReport = latestAudit
          ? await invoke<HumanAuditReport>("load_audit_report", { auditId: latestAudit.auditId, projectRoot })
          : null;

        if (!cancelled) {
          setState({ auditIndex, auditReport, error: null, outputIndex, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setState({
            auditIndex: null,
            auditReport: null,
            error: error instanceof Error ? error.message : String(error),
            outputIndex: null,
            source: "unavailable",
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

  return (
    <main className="v16-login-stage v16-login-shell" data-agentflow-screen="login">
      <header className="v16-login-titlebar" aria-label="登录窗口">
        <WindowDots />
        <span className="v16-login-state">未登录</span>
      </header>
      <section className="v16-login-content" aria-label="连接大模型入口">
        <h1>连接大模型入口</h1>
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

        {stepTitle === "认识智能体" ? (
          <section className="v16-first-run-body v16-agent-brief">
            <AgentBrief className="spec" title="需求助手" value="确认需求 / 整理计划 / 生成任务" />
            <AgentBrief className="build" title="执行助手" value="任务编排 · 执行改动 · 写回结果" />
            <AgentBrief className="audit" title="审计助手" value="审计交付 / 核对证据 / 生成报告" />
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
              content={`请基于当前项目帮我处理：${selectedIntent}\n先确认需求，再生成可交给 Codex 的任务包。`}
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
  children,
  connectedProvider,
  inspector,
  onPageChange,
  projectName,
  projectRoot,
  statusBar,
  toolbar,
}: {
  activePage: AppPage;
  children: ReactNode;
  connectedProvider: Provider;
  inspector: ReactNode;
  onPageChange: (page: AppPage) => void;
  projectName: string;
  projectRoot: string | null;
  statusBar: ReactNode;
  toolbar: ReactNode;
}) {
  return (
    <AppFrame className="v16-app" data-agentflow-ux="v16">
      <TitleBar projectName={projectName} projectRoot={projectRoot} />
      <ProjectTree activePage={activePage} onPageChange={onPageChange} projectName={projectName} />
      <section className={inspector ? "v16-workspace with-inspector" : "v16-workspace"}>
        {toolbar}
        <section className="v16-main-content">{children}</section>
        {inspector}
      </section>
      {statusBar}
    </AppFrame>
  );
}

function TitleBar({ projectName }: { projectName: string; projectRoot: string | null }) {
  return (
    <TopBar className="v16-titlebar">
      <WindowDots />
      <div className="v16-titlebar-project">
        <strong>{projectName}</strong>
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
  onPageChange,
  projectName,
}: {
  activePage: AppPage;
  onPageChange: (page: AppPage) => void;
  projectName: string;
}) {
  return (
    <Sidebar className="v16-project-tree" aria-label="项目导航">
      <header>
        <span className="v16-tree-mark" aria-hidden="true">
          AF
        </span>
        <div>
          <strong>{projectName}</strong>
          <span>本地项目</span>
        </div>
      </header>
      <nav>
        {pages.map((page) => {
          const Icon = page.icon;
          return (
            <button
              className={page.id === activePage ? "active" : ""}
              key={page.id}
              onClick={() => onPageChange(page.id)}
              type="button"
            >
              <Icon size={17} />
              <span>{page.label}</span>
            </button>
          );
        })}
      </nav>
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

function ProjectHomePage({
  nextStep,
  onOpenAudit,
  onOpenDelivery,
  onOpenFiles,
  onOpenTasks,
  onCheckWriteback,
  outputStatusState,
  projectPanelState,
  projectRoot,
  selectedTask,
  stateStatusState,
}: {
  nextStep: NextStepViewModel;
  onOpenAudit: () => void;
  onOpenDelivery: () => void;
  onOpenFiles: () => void;
  onOpenTasks: () => void;
  onCheckWriteback: () => void;
  outputStatusState: OutputStatusState;
  projectPanelState: ProjectPanelState;
  projectRoot: string | null;
  selectedTask: V1Issue | null;
  stateStatusState: StateStatusState;
}) {
  const panelStatus = projectPanelState.status;
  const manifest = projectPanelState.manifest;
  const outputSummary = outputStatusState.status?.summary;

  return (
    <section className="v16-page v16-home-page" data-agentflow-page="workbench">
      <PageHeader
        description="查看下一步、当前任务和最近活动。"
        title="项目工作台"
      />
      <NextStepCard nextStep={nextStep} onOpenAudit={onOpenAudit} onOpenTasks={onOpenTasks} />

      <section className="v16-panel-grid" aria-label="今日待处理">
        <MetricCard label="待确认" value={stateStatusState.status?.blockers.length ?? 0} detail="需要人类确认" />
        <MetricCard label="可交给 Codex" value={selectedTask ? 1 : 0} detail="任务包就绪" />
        <MetricCard label="等待写回" value={outputSummary?.incompleteDeliveries ?? 0} detail="检查 output" />
        <MetricCard label="待审计" value={outputSummary?.releaseDeliveries ?? 0} detail="可请求审计" />
      </section>

      <Panel className="v16-current-work-panel" title="当前任务和最近活动">
        <div className="v16-shortcut-list">
          <button onClick={onOpenTasks} type="button">
            <strong>{selectedTask?.title ?? "还没有任务"}</strong>
            <span>{selectedTask ? `${selectedTask.id} · ${displayStatusLabelZh(selectedTask.displayStatus)}` : "请先确认需求，生成任务。"}</span>
          </button>
          <button onClick={onOpenDelivery} type="button">
            <strong>交付结果</strong>
            <span>{outputSummary?.releaseDeliveries ?? 0} 个交付，{outputSummary?.evidence ?? 0} 份证据</span>
          </button>
          <button onClick={onOpenAudit} type="button">
            <strong>审计报告</strong>
            <span>{outputSummary?.audits ?? 0} 个审计，交付完成后可以请求审计。</span>
          </button>
        </div>
      </Panel>

      <Panel
        className="v16-project-summary"
        description="项目现场只展示人能判断下一步的摘要。内部 JSON 在高级页查看。"
        title="项目现场摘要"
      >
        <div className="v16-summary-grid">
          <MetricCard label="文件数" value={panelStatus?.fileCount ?? manifest?.sourceFiles ?? 0} />
          <MetricCard label="语言数" value={manifest?.languages.length ?? 0} />
          <MetricCard label="诊断" value={panelStatus?.degradedReasons?.length ?? 0} />
          <MetricCard label="测试" value={manifest?.testFiles ?? 0} />
        </div>
        <CompactTable
          columns={[
            { key: "label", label: "项目现场", render: (row) => row.label },
            { key: "value", label: "状态", render: (row) => row.value },
          ]}
          rows={[
            { id: "root", label: "项目", value: projectRoot ?? "未选择项目" },
            { id: "git", label: "Git 状态", value: "仅本地" },
            { id: "context", label: "上下文包", value: projectPanelState.latestContextPack ? "已生成" : "等待需要时生成" },
            { id: "scan", label: "最近索引时间", value: panelStatus?.updatedAt ? formatTimestamp(panelStatus.updatedAt) : "未记录" },
          ]}
        />
      </Panel>

      <CompanionShell
        onCheckWriteback={onCheckWriteback}
        onOpenFiles={onOpenFiles}
        onOpenTasks={onOpenTasks}
        selectedTask={selectedTask}
      />
    </section>
  );
}

function NextStepCard({
  nextStep,
  onOpenAudit,
  onOpenTasks,
}: {
  nextStep: NextStepViewModel;
  onOpenAudit: () => void;
  onOpenTasks: () => void;
}) {
  const primaryAction = nextStep.action === "请求人工审计" ? onOpenAudit : onOpenTasks;
  return (
    <Panel className="v16-next-step-card" title={nextStep.title} tone={nextStep.status === "warning" ? "warning" : "neutral"}>
      <div className="v16-next-step-content">
        <StatusBadge status={nextStep.status}>{nextStep.status === "ready" ? "就绪" : "提醒"}</StatusBadge>
        <p>{nextStep.description}</p>
        <small>{nextStep.reason}</small>
      </div>
      <ActionBar>
        <ActionButton onClick={primaryAction} size="lg" variant="primary">
          {nextStep.action}
        </ActionButton>
      </ActionBar>
    </Panel>
  );
}

function InspectorPanel({ nextStep, selectedTask }: { nextStep: NextStepViewModel; selectedTask: V1Issue | null }) {
  return (
    <aside className="v16-inspector-panel" aria-label="下一步详情">
      <header>
        <p className="v16-kicker">下一步详情</p>
        <h2>{nextStep.title}</h2>
      </header>
      <p>{nextStep.description}</p>
      <dl>
        <div>
          <dt>关联对象</dt>
          <dd>{selectedTask?.id ?? "当前项目"}</dd>
        </div>
        <div>
          <dt>阻断原因</dt>
          <dd>{nextStep.status === "warning" ? nextStep.reason : "无阻断"}</dd>
        </div>
      </dl>
    </aside>
  );
}

function TasksPage({
  actionFeedback,
  actions,
  copyState,
  handedOff,
  onTaskAction,
  onSelectTask,
  selectedTask,
  tasks,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  handedOff: boolean;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  onSelectTask: (taskId: string) => void;
  selectedTask: V1Issue | null;
  tasks: V1Issue[];
}) {
  return (
    <section className="v16-page v16-tasks-page" data-agentflow-page="tasks">
      <TaskList
        actionFeedback={actionFeedback}
        actions={actions}
        copyState={copyState}
        handedOff={handedOff}
        onSelectTask={onSelectTask}
        onTaskAction={onTaskAction}
        selectedTask={selectedTask}
        tasks={tasks}
      />
    </section>
  );
}

function TaskList({
  actionFeedback,
  actions,
  copyState,
  handedOff,
  onSelectTask,
  onTaskAction,
  selectedTask,
  tasks,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  handedOff: boolean;
  onSelectTask: (taskId: string) => void;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  selectedTask: V1Issue | null;
  tasks: V1Issue[];
}) {
  return (
    <div className="v16-task-list-layout" aria-label="任务列表">
      <aside className="v16-list-pane v16-task-queue-pane" aria-label="任务队列">
        <header>
          <h2>任务队列</h2>
          <span>{tasks.length} 项</span>
        </header>
        <div className="v16-task-queue-items">
          {tasks.map((task) => (
            <button
              className={task.id === selectedTask?.id ? "v16-task-queue-row active" : "v16-task-queue-row"}
              key={task.id}
              onClick={() => onSelectTask(task.id)}
              title={`${task.id} ${task.title}`}
              type="button"
            >
              <strong>{task.id}</strong>
              <span className="v16-task-queue-title">{task.title}</span>
              <StatusBadge status={statusChipForDisplayStatus(task.displayStatus)}>
                {displayStatusLabelZh(task.displayStatus)}
              </StatusBadge>
              <small className="v16-task-risk">风险：{displayRiskLabelZh(task.riskLevel)}</small>
            </button>
          ))}
        </div>
      </aside>
      <TaskDetail
        actionFeedback={actionFeedback}
        actions={actions}
        copyState={copyState}
        handedOff={handedOff}
        onTaskAction={onTaskAction}
        task={selectedTask}
      />
    </div>
  );
}

function TaskDetail({
  actionFeedback,
  actions,
  copyState,
  handedOff,
  onTaskAction,
  task,
}: {
  actionFeedback: string | null;
  actions: TaskInteractionAction[];
  copyState: ButtonInteractionState;
  handedOff: boolean;
  onTaskAction: (action: TaskInteractionAction, task: V1Issue) => void;
  task: V1Issue | null;
}) {
  if (!task) {
    return (
      <aside className="v16-detail-pane">
        <p>还没有任务。请先确认需求，生成任务。</p>
      </aside>
    );
  }

  return (
    <aside className="v16-detail-pane" aria-label="任务详情">
      <header>
        <p className="v16-kicker">{task.id}</p>
        <h2>{task.title}</h2>
        <div className="v16-detail-badges">
          <StatusBadge status={statusChipForDisplayStatus(task.displayStatus)}>
            {displayStatusLabelZh(task.displayStatus)}
          </StatusBadge>
          <RiskBadge risk={task.riskLevel || "normal"} />
        </div>
      </header>
      <div className="v16-detail-document">
        <DescriptionList
          items={[
            ["智能体", "执行助手"],
            ["状态", displayStatusLabelZh(task.displayStatus)],
            ["风险", displayRiskLabelZh(task.riskLevel)],
            ["交给 Codex", handedOff ? "已做本地标记" : "未标记"],
            ["来源规格", task.projectId ?? "已确认规格"],
          ]}
        />
        <SectionList title="范围" items={task.scope} />
        <SectionList title="非目标" items={task.nonGoals} />
        <SectionList title="验收标准" items={task.acceptanceCriteria} />
        <SectionList title="相关文件" items={task.allowedFiles} />
        <SectionList title="验证命令" items={task.validationCommands} />
        <SectionList title="证据要求" items={task.evidenceRequired} />
        <CopyableCodeBlock content={buildCodexHandoff(task)} maxHeight={210} title="Codex 任务包" />
      </div>
      {actionFeedback ? <p className="v16-feedback">{actionFeedback}</p> : null}
      <ActionBar sticky>
        {actions.map((action, index) => (
          <ActionButton
            disabled={action === "readonly"}
            key={action}
            loading={action === "copy-handoff" && copyState === "loading"}
            onClick={() => onTaskAction(action, task)}
            variant={index === 0 && action !== "readonly" ? "primary" : "secondary"}
          >
            {action === "copy-handoff" && copyState === "success" ? "已复制" : taskActionLabel(action)}
          </ActionButton>
        ))}
      </ActionBar>
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
      <FileBrowser />
      <FileReader />
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

function FileBrowser() {
  return null;
}

function FileReader() {
  return null;
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
  const deliveries = outputBundle.outputIndex?.releaseDeliveries ?? [];
  const evidence = outputBundle.outputIndex?.evidence ?? [];
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
        <span>{deliveries.length}</span>
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
              <strong>{delivery.runId}</strong>
              <span>{delivery.issueId || "未记录任务"} · {artifactStatusLabel(delivery.status)}</span>
              <small>{formatTimestamp(delivery.updatedAt)}</small>
            </button>
          ))}
        </div>
      ) : (
        <p className="v16-empty-text">暂无 Codex 写回结果。</p>
      )}
    </aside>
  );
}

function DeliveryDetail({
  delivery,
  evidence,
  onOpenAudit,
  outputStatusState,
  selectedTask,
}: {
  delivery: OutputIndexEntry | null;
  evidence: OutputIndexEntry[];
  onOpenAudit: () => void;
  outputStatusState: OutputStatusState;
  selectedTask: V1Issue | null;
}) {
  return (
    <section className="v16-detail-pane" aria-label="交付详情">
      <header>
        <p className="v16-kicker">交付摘要</p>
        <h2>{delivery?.runId ?? "还没有交付材料"}</h2>
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
        <SectionList title="变更文件" items={selectedTask?.allowedFiles ?? ["等待 Codex 写回变更文件。"]} />
        <SectionList title="验证命令" items={selectedTask?.validationCommands ?? ["等待验证命令。"]} />
        <SectionList title="验证结果" items={[delivery ? "浏览器预览：验证记录已回填。" : "等待写回。"]} />
        <SectionList title="证据文件" items={evidence.map((item) => item.path)} />
        <SectionList title="交付记录" items={[delivery?.path ?? "暂无交付记录。"]} />
        <SectionList title="越界检查" items={["普通页面只展示摘要；原始 JSON 在高级页查看。"]} />
      </div>
      <ActionBar sticky>
        <ActionButton disabled={!delivery} onClick={onOpenAudit} variant="primary">
          请求审计
        </ActionButton>
        <ActionButton variant="secondary">查看证据</ActionButton>
      </ActionBar>
    </section>
  );
}

function AuditPage({
  onAuditRequested,
  onSelectAudit,
  outputBundle,
  outputStatusState,
  projectRoot,
  selectedAuditId,
}: {
  onAuditRequested: () => void;
  onSelectAudit: (auditId: string) => void;
  outputBundle: OutputBundleState;
  outputStatusState: OutputStatusState;
  projectRoot: string | null;
  selectedAuditId: string | null;
}) {
  const audits = outputBundle.auditIndex?.audits ?? [];
  const auditInteractionState = buildAuditInteractionState(audits, selectedAuditId);
  const selectedReport =
    outputBundle.auditReport?.audit.auditId === auditInteractionState.selectedAuditId ? outputBundle.auditReport : null;
  return (
    <section className="v16-page v16-audit-page" data-agentflow-page="audit">
      <div className="v16-split-page">
        <AuditList
          audits={audits}
          onSelectAudit={onSelectAudit}
          selectedAuditId={auditInteractionState.selectedAuditId}
        />
        <AuditReport report={selectedReport} selectedAudit={auditInteractionState.selectedAudit} />
      </div>
      <OutputAuditPanel
        onAuditRequested={onAuditRequested}
        outputStatusState={outputStatusState}
        projectRoot={projectRoot}
      />
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
        <span>{audits.length}</span>
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
              <strong>{audit.auditId}</strong>
              <span>{artifactStatusLabel(audit.status)} · {audit.requestedBy}</span>
              <small>{formatTimestamp(audit.requestedAt)}</small>
            </button>
          ))}
        </div>
      ) : (
        <p className="v16-empty-text">还没有请求人工审计。</p>
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
  const findings = Array.isArray(report?.findings)
    ? (report.findings as Array<{ id?: string; severity?: string; summary?: string }>)
    : [];

  return (
    <section className="v16-detail-pane" aria-label="审计报告详情">
      <header>
        <p className="v16-kicker">审计报告</p>
        <h2>{selectedAudit?.auditId ?? report?.audit.auditId ?? "未请求审计"}</h2>
        <StatusBadge status={selectedAudit || report ? "warning" : "idle"}>
          {artifactStatusLabel(selectedAudit?.status ?? report?.audit.status ?? "未请求")}
        </StatusBadge>
      </header>
      <div className="v16-detail-document">
        <SectionList title="审计结论" items={[report?.reportMarkdown.split("\n").slice(0, 3).join(" ") || "选择交付并填写原因后可请求人工审计。"]} />
        <SectionList
          title="发现项"
          items={findings.length ? findings.map((finding) => `${finding.severity ?? "info"}：${finding.summary ?? finding.id ?? "发现项"}`) : ["暂无发现项。"]}
        />
        <JsonSummary title="证据映射" value={report?.evidenceMap ?? { evidence: [], releaseDelivery: [] }} />
        <JsonSummary title="追溯关系" value={report?.traceability ?? { spec: "waiting", issue: "waiting", delivery: "waiting" }} />
        <SectionList title="范围检查" items={["对照规格、任务、交付和证据。"]} />
        <SectionList title="验证检查" items={["检查验证命令是否记录并通过。"]} />
        <SectionList title="处理建议" items={["建议：补充证据", "建议：返工", "建议：接受"]} />
        <SectionList title="当前版本限制" items={["这里只读展示建议，不写接受 / 返工 / 补证据状态。"]} />
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
  stateStatusState: StateStatusState;
  workspaceData: WorkspaceDataState;
}) {
  const categories = [
    { id: "state", label: "状态", value: stateStatusState },
    { id: "panel", label: "面板", value: projectPanelState },
    { id: "input", label: "输入", value: inputStatusState },
    { id: "execute", label: "执行", value: executeStatusState },
    { id: "output", label: "输出", value: { outputBundle, outputStatusState } },
    { id: "audit", label: "审计", value: outputBundle.auditReport },
    { id: "settings", label: "设置", value: { agentManualState, projectFilesState, workspaceData } },
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
      <DesignSystemPreview />
    </section>
  );
}

function AdvancedStateViewer({
  categories,
  onSelectCategory,
  selectedCategory,
}: {
  categories: Array<{ id: string; label: string; value: unknown }>;
  onSelectCategory: (categoryId: string) => void;
  selectedCategory: { id: string; label: string; value: unknown };
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
      </section>
      <JsonReader value={selectedCategory.value} />
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
  agentStatusItems,
  appInteractionState,
  connectedProvider,
  projectName,
  projectRoot,
  stateStatus,
}: {
  agentStatusItems: AgentStatusChannelItem[];
  appInteractionState: AppInteractionState;
  connectedProvider: Provider;
  projectName: string;
  projectRoot: string | null;
  stateStatus: StateStatusState["status"];
}) {
  return (
    <FoundationStatusBar className="v16-status-bar" aria-label="工作流状态通道">
      <section>
        <StatusDot status={projectRoot ? "ready" : "idle"} />
        <span>{projectRoot ? "ready" : "waiting"}</span>
        <strong>{projectName}</strong>
        <span>
          <GitBranch size={13} /> local-only
        </span>
      </section>
      <section className="v16-status-channel">
        <AgentStatusBar items={agentStatusItems} />
      </section>
      <section>
        <span>{workflowStageText(stateStatus?.currentStage)}</span>
        <span>{connectedProvider}</span>
        <span>{lifecycleLabel(appInteractionState.lifecycle)}</span>
        <span>⌘K</span>
      </section>
    </FoundationStatusBar>
  );
}

function CompanionShell({
  onCheckWriteback,
  onOpenFiles,
  onOpenTasks,
  selectedTask,
}: {
  onCheckWriteback: () => void;
  onOpenFiles: () => void;
  onOpenTasks: () => void;
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
        <h2>协作模式</h2>
        <span>{selectedTask ? "等待 Codex 写回" : "等待任务"}</span>
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
            ? "等待 Codex 写回。请确认任务包已经粘贴，然后扫描 .agentflow/output。"
            : "当前没有可交付给执行助手的任务。"}
        </span>
      </article>
      <ActionBar>
        <ActionButton disabled={!selectedTask} onClick={onCheckWriteback} variant="secondary">
          检查写回
        </ActionButton>
        <ActionButton disabled={!selectedTask} onClick={onOpenTasks} variant="primary">
          复制任务包
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

function CompactTable<T extends { id: string }>({
  columns,
  rows,
}: {
  columns: Array<{ key: string; label: string; render: (row: T) => ReactNode }>;
  rows: T[];
}) {
  return (
    <div className="v16-compact-table" role="table">
      <div className="v16-compact-table-row header" role="row">
        {columns.map((column) => (
          <span key={column.key} role="columnheader">
            {column.label}
          </span>
        ))}
      </div>
      {rows.map((row) => (
        <div className="v16-compact-table-row" key={row.id} role="row">
          {columns.map((column) => (
            <span key={column.key} role="cell">
              {column.render(row)}
            </span>
          ))}
        </div>
      ))}
    </div>
  );
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

function SectionList({ items, title }: { items: string[]; title: string }) {
  return (
    <Section className="v16-section-list" title={title}>
      <ul>
        {(items.length ? items : ["暂无记录。"]).map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </Section>
  );
}

function JsonSummary({ title, value }: { title: string; value: unknown }) {
  return (
    <section className="v16-json-summary">
      <h3>{title}</h3>
      <pre>{JSON.stringify(value, null, 2)}</pre>
    </section>
  );
}

const displayStatusColumns: Array<{ id: IssueDisplayStatus; label: string }> = [
  { id: "backlog", label: "待确认" },
  { id: "ready", label: "可交给 Codex" },
  { id: "in-progress", label: "等待写回" },
  { id: "review", label: "待审计" },
  { id: "done", label: "已完成" },
  { id: "cancel", label: "已取消" },
];

const displayStatusOrder = new Map(displayStatusColumns.map((column, index) => [column.id, index]));

function buildTaskItems(
  inputIssues: InputIssue[],
  issueStatusIndex: IssueStatusIndex | null,
  projectViewModel: ProjectMilestoneIssueViewModelSnapshot | null,
  workbench: WorkbenchSnapshot | null,
): V1Issue[] {
  if (inputIssues.length) {
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
  return {
    acceptanceCriteria: issue.acceptanceCriteria,
    allowedFiles: issue.scope,
    boundary: issue.nonGoals,
    codexInstructions: issue.validationHints,
    dependencies: issue.relations?.blockedBy ?? [],
    displayStatus,
    evidenceRequired: issue.acceptanceCriteria,
    forbiddenFiles: [".agentflow/*", ".codex/*", "agent-artifacts/*"],
    goal: issue.summary || issue.title,
    id: issue.issueId,
    milestoneId: null,
    nonGoals: issue.nonGoals,
    projectId: issue.projectId ?? null,
    rawStatus: issue.status,
    riskLevel: issue.riskLevel || indexed?.riskLevel || "normal",
    scope: issue.scope,
    status: issue.status,
    title: issue.title,
    validationCommands: issue.validationHints,
  };
}

function issueContractToV1Issue(issue: IssueContract): V1Issue {
  return {
    acceptanceCriteria: issue.evidenceRequirements,
    allowedFiles: issue.context.files,
    boundary: issue.nonGoals,
    codexInstructions: issue.executionPlan,
    dependencies: [],
    displayStatus: displayStatusFromLegacyStatus(issue.status),
    evidenceRequired: issue.evidenceRequirements,
    forbiddenFiles: [".agentflow/*", ".codex/*", "agent-artifacts/*"],
    goal: issue.intent,
    id: issue.id,
    milestoneId: null,
    nonGoals: issue.nonGoals,
    projectId: null,
    rawStatus: issue.status,
    riskLevel: "normal",
    scope: issue.scope,
    status: issue.status,
    title: issue.title,
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

function sortTasksByDisplayStatus(tasks: V1Issue[]) {
  return [...tasks].sort((left, right) => {
    const leftOrder = displayStatusOrder.get(left.displayStatus ?? "backlog") ?? 0;
    const rightOrder = displayStatusOrder.get(right.displayStatus ?? "backlog") ?? 0;
    return leftOrder - rightOrder || left.id.localeCompare(right.id);
  });
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
    done: "ready",
    "in-progress": "working",
    ready: "ready",
    review: "warning",
  };
  return chips[status];
}

function displayRiskLabelZh(risk?: string | null) {
  const normalized = (risk ?? "normal").toLowerCase();
  if (normalized === "low") {
    return "低";
  }
  if (normalized === "medium" || normalized === "med") {
    return "中";
  }
  if (normalized === "high") {
    return "高";
  }
  return "普通";
}

function taskActionsForRow(task: V1Issue) {
  return taskActionsForStatus(task.displayStatus);
}

function findDeliveryForTask(deliveries: OutputIndexEntry[], taskId: string) {
  return [...deliveries]
    .reverse()
    .find((delivery) => delivery.issueId === taskId || delivery.runId.includes(taskId)) ?? null;
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
      action: "请求人工审计",
      description: "Codex 已经返回交付材料，可以请求人工审计。",
      reason: "交付页已有交付、证据和验证摘要。",
      status: "ready",
      title: "可以审计交付结果",
    };
  }

  if ((inputStatus?.summary.approvedSpecs ?? 0) === 0) {
    return {
      action: "继续整理规格",
      description: "还不能交给 Codex。原因是：这个需求还没有确认成规格。",
      reason: "Spec Agent 需要先整理需求，再由人确认。",
      status: "warning",
      title: "先确认需求",
    };
  }

  if (selectedTask) {
    return {
      action: "复制 Codex 指令",
      description: "这个任务已经有已确认规格和任务合同。",
      reason: selectedTask.id,
      status: "ready",
      title: "可以交给 Codex 了",
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
  return [
    `# ${task.title}`,
    "",
    `任务：${task.id}`,
    `风险：${displayRiskLabelZh(task.riskLevel)}`,
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
    ...task.forbiddenFiles.map((item) => `- ${item}`),
    "",
    "## 验证命令",
    ...task.validationCommands.map((item) => `- ${item}`),
    "",
    "## 交付要求",
    ...task.evidenceRequired.map((item) => `- ${item}`),
  ].join("\n");
}

function buildNextActionLabel(action: string) {
  const labels: Record<string, string> = {
    "start-new-input": "告诉 Agent 你想做什么",
  };
  return labels[action] ?? action;
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
    pending: "待处理",
    ready: "就绪",
    requested: "已请求",
    review: "待审计",
    validated: "已验证",
    waiting: "等待",
  };
  if (!status) {
    return "未记录";
  }
  return labels[status.toLowerCase()] ?? status;
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
  return pages.find((item) => item.id === page)?.label ?? "工作台";
}

function workflowStageText(stage?: string | null) {
  const labels: Record<string, string> = {
    "audit-completed": "审计完成",
    "audit-requested": "审计已请求",
    "delivery-ready": "交付可审计",
    "execute-ready": "可交给 Codex",
    "input-ready": "需求等待确认",
    "workspace-ready": "项目已准备好",
  };
  return stage ? labels[stage] ?? stage : "等待状态";
}

function advancedCategorySummary(categoryId: string) {
  const summaries: Record<string, string> = {
    audit: "展示审计索引和报告快照。这里不接受、返工或补证据。",
    execute: "展示执行状态快照。这里不继续执行，不清理锁。",
    input: "展示需求和 Issue 状态快照。普通页面只展示人能读懂的摘要。",
    output: "展示证据、交付和审计输出摘要。",
    panel: "展示项目现场读取结果和上下文包摘要。",
    settings: "展示本地设置、文件阅读器和工作台数据源状态。",
    state: "展示全局派生状态、门禁、阻断和下一步动作。",
  };
  return summaries[categoryId] ?? "这里展示开发者调试信息。普通页面不显示原始 JSON。";
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
