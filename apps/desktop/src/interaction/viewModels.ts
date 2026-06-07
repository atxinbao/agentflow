import type { AuditIndexEntry, IssueDisplayStatus, OutputIndexEntry, V1Issue } from "../types";

export type AppLifecycleState =
  | "not-authenticated"
  | "first-run"
  | "project-loading"
  | "workspace-ready"
  | "workspace-blocked"
  | "error";

export type PageInteractionState = "loading" | "empty" | "ready" | "blocked" | "error" | "stale";
export type ButtonInteractionState = "enabled" | "disabled" | "loading" | "success" | "error";

export type AppInteractionState = {
  activePage: string;
  lifecycle: AppLifecycleState;
  onboardingComplete: boolean;
  projectRoot: string | null;
  providerConnected: boolean;
};

export type WorkspaceInteractionState = {
  status: PageInteractionState;
  nextAction: string;
};

export type TaskInteractionAction =
  | "view-requirement"
  | "copy-handoff"
  | "mark-handed-off"
  | "check-writeback"
  | "view-delivery"
  | "view-audit"
  | "readonly";

export type TaskInteractionState = {
  actions: TaskInteractionAction[];
  empty: boolean;
  selectedTask: V1Issue | null;
  selectedTaskId: string | null;
  status: PageInteractionState;
};

export type FileInteractionState = {
  readonly: true;
  status: PageInteractionState;
};

export type DeliveryInteractionState = {
  empty: boolean;
  selectedDelivery: OutputIndexEntry | null;
  selectedDeliveryRunId: string | null;
  status: PageInteractionState;
};

export type AuditInteractionState = {
  empty: boolean;
  selectedAudit: AuditIndexEntry | null;
  selectedAuditId: string | null;
  status: PageInteractionState;
};

export type AdvancedInteractionState = {
  readonly: true;
  selectedCategoryId: string;
  status: PageInteractionState;
};

export type CompanionInteractionState = {
  currentTaskId: string | null;
  status: PageInteractionState;
};

export function buildAppInteractionState({
  activePage,
  hasError,
  onboardingComplete,
  projectLoading,
  projectRoot,
  providerConnected,
  workspaceBlocked,
}: {
  activePage: string;
  hasError: boolean;
  onboardingComplete: boolean;
  projectLoading: boolean;
  projectRoot: string | null;
  providerConnected: boolean;
  workspaceBlocked: boolean;
}): AppInteractionState {
  let lifecycle: AppLifecycleState = "workspace-ready";
  if (!providerConnected) {
    lifecycle = "not-authenticated";
  } else if (!onboardingComplete) {
    lifecycle = "first-run";
  } else if (projectLoading) {
    lifecycle = "project-loading";
  } else if (hasError) {
    lifecycle = "error";
  } else if (workspaceBlocked) {
    lifecycle = "workspace-blocked";
  }

  return {
    activePage,
    lifecycle,
    onboardingComplete,
    projectRoot,
    providerConnected,
  };
}

export function buildTaskInteractionState(tasks: V1Issue[], selectedTaskId: string | null): TaskInteractionState {
  const selectedTask = tasks.find((task) => task.id === selectedTaskId) ?? null;
  return {
    actions: selectedTask ? taskActionsForTask(selectedTask) : [],
    empty: tasks.length === 0,
    selectedTask,
    selectedTaskId: selectedTask?.id ?? null,
    status: tasks.length ? "ready" : "empty",
  };
}

export function pickTaskId(tasks: V1Issue[], currentTaskId: string | null, activeIssueId?: string | null) {
  if (!tasks.length) {
    return null;
  }
  if (currentTaskId && tasks.some((task) => task.id === currentTaskId)) {
    return currentTaskId;
  }
  if (activeIssueId && tasks.some((task) => task.id === activeIssueId)) {
    return activeIssueId;
  }
  return (
    tasks.find((task) => task.displayStatus === "in-progress")?.id ??
    tasks.find((task) => task.displayStatus === "ready")?.id ??
    tasks[0].id
  );
}

export function taskActionsForTask(task: V1Issue): TaskInteractionAction[] {
  if (task.issueCategory === "audit") {
    const actions: Record<IssueDisplayStatus, TaskInteractionAction[]> = {
      backlog: ["view-requirement"],
      cancel: ["readonly"],
      done: ["view-audit"],
      "in-progress": ["copy-handoff", "view-audit"],
      ready: ["copy-handoff"],
      review: ["view-audit"],
    };
    return actions[task.displayStatus ?? "backlog"];
  }

  return taskActionsForStatus(task.displayStatus);
}

function taskActionsForStatus(status: IssueDisplayStatus = "backlog"): TaskInteractionAction[] {
  const actions: Record<IssueDisplayStatus, TaskInteractionAction[]> = {
    backlog: ["view-requirement"],
    cancel: ["readonly"],
    done: ["view-delivery", "view-audit"],
    "in-progress": ["mark-handed-off", "check-writeback"],
    ready: ["copy-handoff"],
    review: ["view-delivery", "view-audit"],
  };
  return actions[status];
}

export function taskActionLabel(action: TaskInteractionAction) {
  const labels: Record<TaskInteractionAction, string> = {
    "check-writeback": "检查写回",
    "copy-handoff": "复制任务包",
    "mark-handed-off": "我已交给执行助手",
    readonly: "只读查看",
    "view-audit": "查看审计",
    "view-delivery": "查看交付",
    "view-requirement": "查看需求",
  };
  return labels[action];
}

export function displayStatusLabelZh(status: IssueDisplayStatus = "backlog") {
  const labels: Record<IssueDisplayStatus, string> = {
    backlog: "待办",
    cancel: "已取消",
    done: "已完成",
    "in-progress": "进行中",
    ready: "就绪",
    review: "待审阅",
  };
  return labels[status];
}

export function buildDeliveryInteractionState(
  deliveries: OutputIndexEntry[],
  selectedDeliveryRunId: string | null,
): DeliveryInteractionState {
  const selectedDelivery =
    deliveries.find((delivery) => delivery.runId === selectedDeliveryRunId) ?? deliveries.at(0) ?? null;
  return {
    empty: deliveries.length === 0,
    selectedDelivery,
    selectedDeliveryRunId: selectedDelivery?.runId ?? null,
    status: deliveries.length ? "ready" : "empty",
  };
}

export function buildAuditInteractionState(audits: AuditIndexEntry[], selectedAuditId: string | null): AuditInteractionState {
  const selectedAudit = audits.find((audit) => audit.auditId === selectedAuditId) ?? audits.at(0) ?? null;
  return {
    empty: audits.length === 0,
    selectedAudit,
    selectedAuditId: selectedAudit?.auditId ?? null,
    status: audits.length ? "ready" : "empty",
  };
}
