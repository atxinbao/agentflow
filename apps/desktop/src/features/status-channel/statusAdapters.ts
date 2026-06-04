import type { ProjectFilesState, ProjectPanelState } from "../project-files";
import type { AgentManualState } from "../agent-manual";
import type { AgentStatusChannelItem, AgentStatusTone } from "./statusTypes";

export function buildAgentStatusItems({
  agentManualState,
  projectFilesState,
  projectPanelState,
}: {
  agentManualState: AgentManualState;
  projectFilesState: ProjectFilesState;
  projectPanelState: ProjectPanelState;
}): AgentStatusChannelItem[] {
  return [
    buildWorkspaceStatus(projectFilesState),
    buildWorksiteStatus(projectPanelState),
    buildAgentManualStatus(agentManualState),
  ];
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
      { label: "AGENTS.md", value: status.agentMd.managed ? "Managed" : "未接管" },
      { label: "Layout", value: status.layout.ready ? "Ready" : "缺失" },
      { label: "Skills", value: `${status.skills.filter((skill) => skill.exists && skill.hashMatches).length}/${status.skillsLock.skillCount}` },
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
