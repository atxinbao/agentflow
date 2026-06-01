import type { ProjectFilesState, ProjectGraphState } from "../project-files";
import type { AgentStatusChannelItem, AgentStatusTone } from "./statusTypes";

export function buildAgentStatusItems({
  projectFilesState,
  projectGraphState,
}: {
  projectFilesState: ProjectFilesState;
  projectGraphState: ProjectGraphState;
}): AgentStatusChannelItem[] {
  return [buildWorkspaceStatus(projectFilesState), buildWorksiteStatus(projectGraphState)];
}

function buildWorkspaceStatus(projectFilesState: ProjectFilesState): AgentStatusChannelItem {
  const source = "Project Workspace Manager V0.2";
  const entryCount = projectFilesState.snapshot?.entries.length ?? 0;
  const selectedPath = projectFilesState.selectedPath ?? projectFilesState.snapshot?.selectedPath ?? "未选择";

  if (projectFilesState.error) {
    return {
      id: "agent-workspace",
      label: "Agent 工作空间",
      status: "failed",
      statusLabel: "异常",
      source,
      metrics: [{ label: "资源", value: entryCount }],
      error: projectFilesState.error,
    };
  }

  if (projectFilesState.source === "loading") {
    return {
      id: "agent-workspace",
      label: "Agent 工作空间",
      status: "working",
      statusLabel: "准备中",
      source,
    };
  }

  if (projectFilesState.snapshot) {
    return {
      id: "agent-workspace",
      label: "Agent 工作空间",
      status: "ready",
      statusLabel: "已就绪",
      source,
      metrics: [
        { label: "资源", value: entryCount },
        { label: "选中", value: selectedPath, title: selectedPath },
      ],
    };
  }

  return {
    id: "agent-workspace",
    label: "Agent 工作空间",
    status: "idle",
    statusLabel: "未就绪",
    source,
  };
}

function buildWorksiteStatus(projectGraphState: ProjectGraphState): AgentStatusChannelItem {
  const source = "002 - Graph V1";
  const graphStatus = projectGraphState.status?.status ?? "missing";
  const languageText = projectGraphState.manifest?.languages.length
    ? projectGraphState.manifest.languages.slice(0, 5).join(" / ")
    : "未记录";

  if (projectGraphState.error) {
    return {
      id: "agent-worksite",
      label: "Agent 工作现场",
      status: "failed",
      statusLabel: "异常",
      source,
      error: projectGraphState.error,
    };
  }

  return {
    id: "agent-worksite",
    label: "Agent 工作现场",
    status: graphStatusTone(graphStatus, projectGraphState.source),
    statusLabel: graphStatusLabel(graphStatus, projectGraphState.source),
    source,
    metrics: [
      { label: "文件", value: projectGraphState.status?.fileCount ?? 0 },
      { label: "符号", value: projectGraphState.status?.symbolCount ?? 0 },
      { label: "关系", value: projectGraphState.status?.relationCount ?? 0 },
      { label: "语言", value: languageText, title: languageText },
    ],
  };
}

function graphStatusTone(status: string, source: ProjectGraphState["source"]): AgentStatusTone {
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

function graphStatusLabel(status: string, source: ProjectGraphState["source"]) {
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
