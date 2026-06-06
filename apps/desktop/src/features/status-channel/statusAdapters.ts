import type { ProjectFilesState, ProjectPanelState } from "../project-files";
import type { AgentManualState } from "../agent-manual";
import type { ExecuteStatusState } from "../execute";
import type { InputStatusState } from "../input";
import type { OutputStatusState } from "../output";
import type { StateStatusState } from "../state";
import type { AgentStatusChannelItem, AgentStatusTone } from "./statusTypes";

export function buildAgentStatusItems({
  agentManualState,
  executeStatusState,
  inputStatusState,
  outputStatusState,
  projectFilesState,
  projectPanelState,
  stateStatusState,
}: {
  agentManualState: AgentManualState;
  executeStatusState: ExecuteStatusState;
  inputStatusState: InputStatusState;
  outputStatusState: OutputStatusState;
  projectFilesState: ProjectFilesState;
  projectPanelState: ProjectPanelState;
  stateStatusState: StateStatusState;
}): AgentStatusChannelItem[] {
  return [
    buildWorkspaceStatus(projectFilesState),
    buildWorkspaceOwnershipStatus(agentManualState),
    buildWorksiteStatus(projectPanelState),
    buildInputStatus(inputStatusState),
    buildExecuteStatus(executeStatusState),
    buildOutputStatus(outputStatusState),
    buildWorkflowStateStatus(stateStatusState),
    buildAgentManualStatus(agentManualState),
  ];
}

function buildWorkflowStateStatus(stateStatusState: StateStatusState): AgentStatusChannelItem {
  const source = "013 - Workflow State / Gate Orchestration V1";
  const status = stateStatusState.status;

  if (stateStatusState.error) {
    return {
      id: "workflow-state",
      label: "工作流状态",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 31,
      error: stateStatusState.error,
    };
  }

  if (stateStatusState.source === "loading") {
    return {
      id: "workflow-state",
      label: "工作流状态",
      status: "working",
      statusLabel: "检查中",
      source,
      priority: 31,
    };
  }

  if (!status) {
    return {
      id: "workflow-state",
      label: "工作流状态",
      status: "idle",
      statusLabel: "未检查",
      source,
      priority: 31,
    };
  }

  return {
    id: "workflow-state",
    label: "工作流状态",
    status: workflowStateTone(status.status, status.currentStage),
    statusLabel: workflowStateLabel(status.status, status.currentStage),
    source,
    priority: 31,
    metrics: [
      { label: "阶段", value: workflowStageLabel(status.currentStage) },
      { label: "下一步", value: status.nextActions.length },
      { label: "阻断", value: status.blockers.length },
      { label: "审计", value: workflowAuditLabel(status.auditStatus) },
      { label: "运行", value: status.activeRunId ?? "无" },
    ],
    error: status.blockers.at(0)?.reason ?? null,
  };
}

function buildOutputStatus(outputStatusState: OutputStatusState): AgentStatusChannelItem {
  const source = "012 - Human-triggered Audit Report V1";
  const status = outputStatusState.status;

  if (outputStatusState.error) {
    return {
      id: "agent-output",
      label: "交付输出",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 29,
      error: outputStatusState.error,
    };
  }

  if (outputStatusState.source === "loading") {
    return {
      id: "agent-output",
      label: "交付输出",
      status: "working",
      statusLabel: "检查中",
      source,
      priority: 29,
    };
  }

  if (!status) {
    return {
      id: "agent-output",
      label: "交付输出",
      status: "idle",
      statusLabel: "未检查",
      source,
      priority: 29,
    };
  }

  return {
    id: "agent-output",
    label: "交付输出",
    status: inputTone(status.status),
    statusLabel: inputStatusLabel(status.status),
    source,
    priority: 29,
    metrics: [
      { label: "证据", value: status.summary.evidence },
      { label: "交付", value: status.summary.releaseDeliveries },
      { label: "审计", value: status.summary.audits },
      {
        label: "未完成",
        value: status.summary.incompleteEvidence + status.summary.incompleteDeliveries,
      },
    ],
    error: status.errors.at(0) ?? null,
  };
}

function buildExecuteStatus(executeStatusState: ExecuteStatusState): AgentStatusChannelItem {
  const source = "010 - Execute Patch / Checkpoint V1";
  const status = executeStatusState.status;

  if (executeStatusState.error) {
    return {
      id: "agent-execute",
      label: "执行流水线",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 28,
      error: executeStatusState.error,
    };
  }

  if (executeStatusState.source === "loading") {
    return {
      id: "agent-execute",
      label: "执行流水线",
      status: "working",
      statusLabel: "检查中",
      source,
      priority: 28,
    };
  }

  if (!status) {
    return {
      id: "agent-execute",
      label: "执行流水线",
      status: "idle",
      statusLabel: "未检查",
      source,
      priority: 28,
    };
  }

  return {
    id: "agent-execute",
    label: "执行流水线",
    status: inputTone(status.status),
    statusLabel: inputStatusLabel(status.status),
    source,
    priority: 28,
    metrics: [
      { label: "Runs", value: status.summary.runs },
      { label: "Active", value: status.summary.activeRuns },
      { label: "Blocked", value: status.summary.blockedRuns },
      { label: "Completed", value: status.summary.completedRuns },
      { label: "Leases", value: status.summary.activeLeases },
    ],
    error: status.errors.at(0) ?? null,
  };
}

function buildInputStatus(inputStatusState: InputStatusState): AgentStatusChannelItem {
  const source = "009 - Input Model V1";
  const status = inputStatusState.status;

  if (inputStatusState.error) {
    return {
      id: "agent-input",
      label: "需求输入",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 25,
      error: inputStatusState.error,
    };
  }

  if (inputStatusState.source === "loading") {
    return {
      id: "agent-input",
      label: "需求输入",
      status: "working",
      statusLabel: "检查中",
      source,
      priority: 25,
    };
  }

  if (!status) {
    return {
      id: "agent-input",
      label: "需求输入",
      status: "idle",
      statusLabel: "未检查",
      source,
      priority: 25,
    };
  }

  return {
    id: "agent-input",
    label: "需求输入",
    status: inputTone(status.status),
    statusLabel: inputStatusLabel(status.status),
    source,
    priority: 25,
    metrics: [
      { label: "Intake", value: status.summary.intake },
      { label: "Draft SPEC", value: status.summary.draftSpecs },
      { label: "Approved SPEC", value: status.summary.approvedSpecs },
      { label: "Projects", value: status.summary.projects },
      { label: "Issues", value: status.summary.issues },
      { label: "High Risk", value: status.summary.highRiskIssues },
    ],
    error: status.errors.at(0) ?? null,
  };
}

function inputTone(status: NonNullable<InputStatusState["status"]>["status"]): AgentStatusTone {
  if (status === "ready" || status === "degraded") {
    return status === "ready" ? "ready" : "warning";
  }
  if (status === "missing") {
    return "warning";
  }
  if (status === "failed" || status === "blocked") {
    return "failed";
  }
  return "idle";
}

function inputStatusLabel(status: NonNullable<InputStatusState["status"]>["status"]) {
  const labels: Record<NonNullable<InputStatusState["status"]>["status"], string> = {
    missing: "缺失",
    ready: "已就绪",
    degraded: "降级",
    failed: "失败",
    blocked: "已阻断",
  };
  return labels[status] ?? status;
}

function workflowStateTone(
  status: NonNullable<StateStatusState["status"]>["status"],
  stage: NonNullable<StateStatusState["status"]>["currentStage"],
): AgentStatusTone {
  if (status === "failed" || status === "blocked" || stage === "failed" || stage === "workspace-blocked") {
    return "failed";
  }
  if (stage === "execute-running" || stage === "audit-running") {
    return "working";
  }
  if (status === "degraded" || status === "missing" || stage === "workspace-missing" || stage === "execute-blocked") {
    return "warning";
  }
  if (status === "ready") {
    return "ready";
  }
  return "idle";
}

function workflowStateLabel(
  status: NonNullable<StateStatusState["status"]>["status"],
  stage: NonNullable<StateStatusState["status"]>["currentStage"],
) {
  if (status === "blocked") {
    return "已阻断";
  }
  if (status === "failed") {
    return "异常";
  }
  return workflowStageLabel(stage);
}

function workflowStageLabel(stage: NonNullable<StateStatusState["status"]>["currentStage"]) {
  const labels: Record<NonNullable<StateStatusState["status"]>["currentStage"], string> = {
    "workspace-missing": "工作区缺失",
    "workspace-blocked": "工作区阻断",
    "workspace-ready": "工作区已就绪",
    "panel-ready": "面板已就绪",
    "input-ready": "输入已就绪",
    "issue-ready": "任务已就绪",
    "execute-ready": "执行已就绪",
    "execute-running": "执行中",
    "execute-blocked": "执行阻断",
    "execute-completed": "执行完成",
    "evidence-ready": "证据已就绪",
    "delivery-ready": "交付已就绪",
    "audit-requested": "审计已请求",
    "audit-running": "审计中",
    "audit-completed": "审计完成",
    failed: "失败",
  };
  return labels[stage] ?? stage;
}

function workflowAuditLabel(status: NonNullable<StateStatusState["status"]>["auditStatus"]) {
  const labels: Record<NonNullable<StateStatusState["status"]>["auditStatus"], string> = {
    "not-requested": "未请求",
    requested: "已请求",
    running: "运行中",
    passed: "通过",
    "passed-with-warnings": "带警告通过",
    failed: "失败",
    cancelled: "已取消",
  };
  return labels[status] ?? status;
}

function buildWorkspaceOwnershipStatus(agentManualState: AgentManualState): AgentStatusChannelItem {
  const source = "008.4.2 - Workspace Ownership Guard V1";
  const ownership = agentManualState.status?.ownership;

  if (agentManualState.error) {
    return {
      id: "workspace-ownership",
      label: "工作区归属",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 15,
      error: agentManualState.error,
    };
  }

  if (agentManualState.source === "loading") {
    return {
      id: "workspace-ownership",
      label: "工作区归属",
      status: "working",
      statusLabel: "检查中",
      source,
      priority: 15,
    };
  }

  if (!ownership) {
    return {
      id: "workspace-ownership",
      label: "工作区归属",
      status: "idle",
      statusLabel: "未检查",
      source,
      priority: 15,
    };
  }

  return {
    id: "workspace-ownership",
    label: "工作区归属",
    status: ownershipTone(ownership.status),
    statusLabel: ownershipLabel(ownership.status),
    source,
    priority: 15,
    metrics: [
      { label: "动作", value: ownershipActionLabel(ownership.recommendedAction) },
      { label: "标记", value: ownership.detectedFiles.length },
      { label: "Warnings", value: ownership.warnings.length },
      { label: "Errors", value: ownership.errors.length },
    ],
    error: ownership.errors.at(0) ?? null,
  };
}

function buildAgentManualStatus(agentManualState: AgentManualState): AgentStatusChannelItem {
  const source = "008.3 - Workflow Directory Blueprint V1";
  const status = agentManualState.status;

  if (agentManualState.error) {
    return {
      id: "agent-manual",
      label: "工作手册",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 30,
      error: agentManualState.error,
    };
  }

  if (agentManualState.source === "loading") {
    return {
      id: "agent-manual",
      label: "工作手册",
      status: "working",
      statusLabel: "检查中",
      source,
      priority: 30,
    };
  }

  if (!status) {
    return {
      id: "agent-manual",
      label: "工作手册",
      status: "idle",
      statusLabel: "未检查",
      source,
      priority: 30,
    };
  }

  return {
    id: "agent-manual",
    label: "工作手册",
    status: agentManualTone(status.status),
    statusLabel: agentManualLabel(status.status),
    source,
    priority: 30,
    metrics: [
      {
        label: "AGENTS.md",
        value: status.agentMd.exists ? (status.agentMd.managed ? "Managed" : "Local") : "缺失",
      },
      { label: "Layout", value: status.layout.ready ? "Ready" : "缺失" },
      { label: "Skills", value: `${status.skills.filter((skill) => skill.exists && skill.hashMatches).length}/${status.skillsLock.skillCount}` },
      { label: "Agent locale", value: status.locale.fallback ? `${status.locale.agentLocale} fallback` : status.locale.agentLocale },
      { label: "Manual language", value: status.locale.manualLanguage },
      { label: "Voice style", value: status.style.styleId },
      { label: "Warnings", value: status.warnings.length },
      { label: "Errors", value: status.errors.length },
    ],
    error: status.errors.at(0) ?? null,
  };
}

function agentManualTone(status: NonNullable<AgentManualState["status"]>["status"]): AgentStatusTone {
  if (status === "ready" || status === "repaired") {
    return "ready";
  }
  if (status === "checking" || status === "repairing") {
    return "working";
  }
  if (status === "degraded" || status === "missing") {
    return "warning";
  }
  if (status === "failed" || status === "blocked") {
    return "failed";
  }
  return "idle";
}

function agentManualLabel(status: NonNullable<AgentManualState["status"]>["status"]) {
  const labels: Record<NonNullable<AgentManualState["status"]>["status"], string> = {
    missing: "缺失",
    checking: "检查中",
    repairing: "修复中",
    ready: "已就绪",
    repaired: "已修复",
    degraded: "降级",
    failed: "失败",
    blocked: "已阻断",
  };
  return labels[status] ?? status;
}

function ownershipTone(status: NonNullable<AgentManualState["status"]>["ownership"]["status"]): AgentStatusTone {
  if (status === "managed-current" || status === "managed-legacy") {
    return "ready";
  }
  if (status === "none" || status === "corrupted") {
    return "warning";
  }
  if (status === "foreign" || status === "blocked") {
    return "failed";
  }
  return "idle";
}

function ownershipLabel(status: NonNullable<AgentManualState["status"]>["ownership"]["status"]) {
  const labels: Record<NonNullable<AgentManualState["status"]>["ownership"]["status"], string> = {
    none: "未创建",
    "managed-current": "已接管",
    "managed-legacy": "旧版接管",
    foreign: "外部目录",
    corrupted: "待修复",
    blocked: "已阻断",
  };
  return labels[status] ?? status;
}

function ownershipActionLabel(action: NonNullable<AgentManualState["status"]>["ownership"]["recommendedAction"]) {
  const labels: Record<NonNullable<AgentManualState["status"]>["ownership"]["recommendedAction"], string> = {
    create: "创建",
    "validate-repair": "检查/修复",
    "migrate-repair": "迁移/修复",
    "ask-user-to-take-over": "等待接管确认",
    stop: "停止",
  };
  return labels[action] ?? action;
}

function buildWorkspaceStatus(projectFilesState: ProjectFilesState): AgentStatusChannelItem {
  const source = "Project Workspace Manager V0.2";
  const entryCount = projectFilesState.snapshot?.entries.length ?? 0;
  const selectedPath = projectFilesState.selectedPath ?? projectFilesState.snapshot?.selectedPath ?? "未选择";

  if (projectFilesState.error) {
    return {
      id: "agent-workspace",
      label: "工作空间",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 10,
      metrics: [{ label: "资源", value: entryCount }],
      error: projectFilesState.error,
    };
  }

  if (projectFilesState.source === "loading") {
    return {
      id: "agent-workspace",
      label: "工作空间",
      status: "working",
      statusLabel: "准备中",
      source,
      priority: 10,
    };
  }

  if (projectFilesState.snapshot) {
    return {
      id: "agent-workspace",
      label: "工作空间",
      status: "ready",
      statusLabel: "已就绪",
      source,
      priority: 10,
      metrics: [
        { label: "资源", value: entryCount },
        { label: "选中", value: selectedPath, title: selectedPath },
      ],
    };
  }

  return {
    id: "agent-workspace",
    label: "工作空间",
    status: "idle",
    statusLabel: "未就绪",
    source,
    priority: 10,
  };
}

function buildWorksiteStatus(projectPanelState: ProjectPanelState): AgentStatusChannelItem {
  const source = "008.4 - Project Panel V1";
  const panelStatus = projectPanelState.status?.status ?? "missing";
  const languageText = projectPanelState.manifest?.languages.length
    ? projectPanelState.manifest.languages.slice(0, 5).join(" / ")
    : "未记录";

  if (projectPanelState.error) {
    return {
      id: "agent-worksite",
      label: "工作现场",
      status: "failed",
      statusLabel: "异常",
      source,
      priority: 20,
      error: projectPanelState.error,
    };
  }

  return {
    id: "agent-worksite",
    label: "工作现场",
    status: panelStatusTone(panelStatus, projectPanelState.source),
    statusLabel: panelStatusLabel(panelStatus, projectPanelState.source),
    source,
    priority: 20,
    metrics: [
      { label: "文件", value: projectPanelState.status?.fileCount ?? 0 },
      { label: "符号", value: projectPanelState.status?.symbolCount ?? 0 },
      { label: "关系", value: projectPanelState.status?.relationCount ?? 0 },
      { label: "语言", value: languageText, title: languageText },
      { label: "Watcher", value: projectPanelState.status?.watcherStatus ?? "未启动" },
      { label: "Backend", value: projectPanelState.status?.watcherBackend ?? "未记录" },
      { label: "Preflight", value: projectPanelState.status?.preflightStatus ?? "未执行" },
      { label: "Protection", value: projectPanelState.status?.protectionStatus ?? "未检查" },
    ],
  };
}

function panelStatusTone(status: string, source: ProjectPanelState["source"]): AgentStatusTone {
  if (source === "loading" || status === "indexing") {
    return "working";
  }
  if (status === "ready") {
    return "ready";
  }
  if (status === "stale" || status === "degraded") {
    return "warning";
  }
  if (status === "failed" || source === "unavailable") {
    return "failed";
  }
  return "idle";
}

function panelStatusLabel(status: string, source: ProjectPanelState["source"]) {
  if (source === "loading") {
    return "建立中";
  }

  const labels: Record<string, string> = {
    missing: "未建立",
    indexing: "建立中",
    ready: "已就绪",
    stale: "需更新",
    failed: "失败",
    degraded: "降级",
  };
  return labels[status] ?? status;
}
