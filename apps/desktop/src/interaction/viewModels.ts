import type {
  AgentRole,
  AuditIndexEntry,
  ExpectedOutputs,
  InputIssue,
  InputIssueRelationEdge,
  InputIssueRelations,
  InputIssueStatus,
  InputProject,
  IssueCategory,
  IssueDisplayStatus,
  IssueStatusIndex,
  IssueStatusIndexEntry,
  OutputIndexEntry,
  V1Issue,
  WorkflowAuditStatus,
} from "../types";

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

export type TaskSelection =
  | {
      kind: "issue";
      issueId: string;
      projectId?: string | null;
    }
  | {
      kind: "project";
      projectId: string;
    }
  | {
      kind: "empty";
    };

export type TaskProjectTreeWarningKind =
  | "duplicate-project-issue"
  | "missing-issue"
  | "missing-project"
  | "missing-relation-issue";

export type TaskProjectTreeWarning = {
  kind: TaskProjectTreeWarningKind;
  message: string;
  issueId?: string;
  projectId?: string;
};

export type TaskProjectTreeCounts = {
  projectCount: number;
  issueCount: number;
  activeIssueCount: number;
  auditIssueCount: number;
  doneIssueCount: number;
};

export type TaskIssueNode = {
  id: string;
  projectId?: string | null;
  title: string;
  summary: string;
  issueCategory: IssueCategory;
  requiredAgentRole: AgentRole;
  status: InputIssueStatus;
  displayStatus: IssueDisplayStatus;
  priority: string;
  executionRisk: string;
  blockedBy: string[];
  blocks: string[];
  latestRunId?: string | null;
  executeStatus?: string | null;
  evidenceStatus: string;
  deliveryStatus: string;
  auditStatus: WorkflowAuditStatus;
  sourceSpecId?: string | null;
  sourceSpecPath?: string | null;
  expectedOutputs: ExpectedOutputs;
  issue: InputIssue;
  warnings: TaskProjectTreeWarning[];
};

export type TaskProjectGroup = {
  id: string;
  title: string;
  summary: string;
  objective?: string | null;
  sourceSpecId?: string | null;
  status: string;
  issues: TaskIssueNode[];
  missingIssueIds: string[];
  counts: TaskProjectTreeCounts;
  project: InputProject;
  warnings: TaskProjectTreeWarning[];
};

export type TaskProjectTreeViewModel = {
  version: "task-project-tree-view-model.v1";
  groups: TaskProjectGroup[];
  ungroupedIssues: TaskIssueNode[];
  selection: TaskSelection;
  counts: TaskProjectTreeCounts;
  warnings: TaskProjectTreeWarning[];
};

export type TaskStatusContract = {
  status: IssueDisplayStatus;
  label: string;
  businessMeaning: string;
  ownerRoleLabel: string;
  stageAction: string;
  stageOutputs: string[];
  nextEntry: string;
  uiSummary: string[];
};

export type BuildTaskProjectTreeViewModelInput = {
  activeIssueId?: string | null;
  issues: InputIssue[];
  issueStatusIndex?: IssueStatusIndex | null;
  projects: InputProject[];
  relations?: InputIssueRelations | null;
};

type ExecutionOrderedTask = {
  id: string;
  dependencies?: string[];
  blockedBy?: string[];
  priority?: string | null;
  displayStatus?: IssueDisplayStatus;
  updatedAt?: number | null;
  createdAt?: number | null;
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
    tasks.find((task) => task.displayStatus === "in_progress")?.id ??
    tasks.find((task) => task.displayStatus === "todo")?.id ??
    tasks[0].id
  );
}

export function buildTaskProjectTreeViewModel({
  activeIssueId,
  issues,
  issueStatusIndex,
  projects,
  relations,
}: BuildTaskProjectTreeViewModelInput): TaskProjectTreeViewModel {
  const warnings: TaskProjectTreeWarning[] = [];
  const projectById = new Map(projects.map((project) => [project.projectId, project]));
  const relationMap = buildTaskRelationMap(issues, relations, warnings);
  const statusByIssueId = new Map((issueStatusIndex?.issues ?? []).map((entry) => [entry.issueId, entry]));
  const nodeById = new Map(
    issues.map((issue) => [
      issue.issueId,
      inputIssueToTaskIssueNode(issue, statusByIssueId.get(issue.issueId), relationMap),
    ]),
  );
  const assignedIssueIds = new Set<string>();
  const groups = projects.map((project) => {
    const groupWarnings: TaskProjectTreeWarning[] = [];
    const issueIds = projectIssueOrder(project, issues);
    const groupIssues: TaskIssueNode[] = [];
    const missingIssueIds: string[] = [];

    issueIds.forEach((issueId) => {
      const node = nodeById.get(issueId);
      if (!node) {
        const warning = taskProjectTreeWarning(
          "missing-issue",
          `Project ${project.projectId} 引用了不存在的 issue ${issueId}。`,
          project.projectId,
          issueId,
        );
        groupWarnings.push(warning);
        warnings.push(warning);
        missingIssueIds.push(issueId);
        return;
      }

      if (assignedIssueIds.has(issueId)) {
        const warning = taskProjectTreeWarning(
          "duplicate-project-issue",
          `任务 ${issueId} 被多个 project 引用，只保留第一次分组。`,
          project.projectId,
          issueId,
        );
        groupWarnings.push(warning);
        warnings.push(warning);
        return;
      }

      assignedIssueIds.add(issueId);
      groupIssues.push(node);
    });

    return {
      counts: taskProjectTreeCounts(groupIssues, 1),
      id: project.projectId,
      issues: sortTasksByExecutionOrder(groupIssues),
      missingIssueIds,
      objective: project.objective ?? null,
      project,
      sourceSpecId: project.sourceSpecId ?? null,
      status: project.status,
      summary: project.summary,
      title: project.title,
      warnings: groupWarnings,
    };
  });

  const ungroupedIssues = sortTasksByExecutionOrder(
    issues
      .filter((issue) => {
        if (assignedIssueIds.has(issue.issueId)) {
          return false;
        }
        if (issue.projectId && !projectById.has(issue.projectId)) {
          const warning = taskProjectTreeWarning(
            "missing-project",
            `任务 ${issue.issueId} 指向不存在的 project ${issue.projectId}，已放入未归属任务。`,
            issue.projectId,
            issue.issueId,
          );
          warnings.push(warning);
          nodeById.get(issue.issueId)?.warnings.push(warning);
        }
        return issue.issueModel === "direct" || !issue.projectId || !projectById.has(issue.projectId);
      })
      .map((issue) => nodeById.get(issue.issueId))
      .filter((node): node is TaskIssueNode => Boolean(node)),
  );
  const allIssues = [...groups.flatMap((group) => group.issues), ...ungroupedIssues];

  return {
    counts: taskProjectTreeCounts(allIssues, groups.length),
    groups,
    selection: pickTaskSelection(groups, ungroupedIssues, activeIssueId),
    ungroupedIssues,
    version: "task-project-tree-view-model.v1",
    warnings,
  };
}

export function taskActionsForTask(task: V1Issue): TaskInteractionAction[] {
  if (task.issueCategory === "audit") {
    const actions: Record<IssueDisplayStatus, TaskInteractionAction[]> = {
      backlog: ["view-requirement"],
      blocked: ["view-requirement"],
      cancel: ["readonly"],
      done: [],
      in_progress: ["copy-handoff"],
      in_review: [],
      todo: ["copy-handoff"],
    };
    return actions[task.displayStatus ?? "backlog"];
  }

  return taskActionsForStatus(task.displayStatus);
}

function taskActionsForStatus(status: IssueDisplayStatus = "backlog"): TaskInteractionAction[] {
  const actions: Record<IssueDisplayStatus, TaskInteractionAction[]> = {
    backlog: ["view-requirement"],
    blocked: ["view-requirement"],
    cancel: ["readonly"],
    done: ["view-delivery"],
    in_progress: ["mark-handed-off", "check-writeback"],
    in_review: ["view-delivery"],
    todo: ["copy-handoff"],
  };
  return actions[status];
}

export function taskActionLabel(action: TaskInteractionAction) {
  const labels: Record<TaskInteractionAction, string> = {
    "check-writeback": "检查写回",
    "copy-handoff": "复制任务包",
    "mark-handed-off": "我已交给执行助手",
    readonly: "只读查看",
    "view-delivery": "查看交付",
    "view-requirement": "查看需求",
  };
  return labels[action];
}

export function displayStatusLabelZh(status: IssueDisplayStatus = "backlog") {
  const labels: Record<IssueDisplayStatus, string> = {
    backlog: "待处理",
    blocked: "已阻断",
    cancel: "已取消",
    done: "已完成",
    in_progress: "正在做",
    in_review: "正在评审",
    todo: "准备开工",
  };
  return labels[status];
}

export function buildTaskStatusContract(task: V1Issue): TaskStatusContract {
  const status = task.displayStatus ?? "backlog";
  const roleLabel = statusOwnerRoleLabel(status, task.issueCategory);
  const validationCount = task.validationCommands.length;
  const outputCount = Object.keys(task.expectedOutputs ?? {}).length;
  const hasDependencies = task.dependencies.length > 0;

  const contracts: Record<IssueDisplayStatus, TaskStatusContract> = {
    backlog: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务已经进入任务池，但还没有进入执行准备。",
      ownerRoleLabel: roleLabel,
      stageAction: "整理任务边界，确认范围、非目标和依赖关系。",
      stageOutputs: [
        "任务合同已生成。",
        hasDependencies ? `前置依赖：${task.dependencies.join("、")}` : "前置依赖：无。",
        task.allowedFiles.length ? "允许变更范围已锁定。" : "允许变更范围待补充。",
      ],
      nextEntry: "先确认任务合同，再进入执行前置检测。",
      uiSummary: [
        "任务已创建。",
        "还没有进入执行。",
      ],
    },
    todo: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务已经满足开工前提，正在等待正式进入执行。",
      ownerRoleLabel: roleLabel,
      stageAction: "执行前置检测，确认合同完整、Context Pack 可读、工作区干净。",
      stageOutputs: [
        task.contextPackPath ? `Context Pack：${task.contextPackPath}` : "Context Pack：等待生成。",
        "当前 run 已准备就绪。",
        validationCount ? `验证命令：${validationCount} 条` : "验证命令：待补充。",
      ],
      nextEntry: "通过前置检测后进入正在做。",
      uiSummary: [
        "可以开工。",
        "等待执行助手接手。",
      ],
    },
    in_progress: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务正在执行，执行助手正在按合同修改代码并做本地验证。",
      ownerRoleLabel: roleLabel,
      stageAction: "完成测试设计、实现改动和沙箱验证。",
      stageOutputs: [
        validationCount ? `验证命令：${validationCount} 条` : "验证命令：待记录。",
        outputCount ? `预期输出：${outputCount} 项` : "预期输出：待记录。",
        "本地验证结果会在进入评审前补齐。",
      ],
      nextEntry: "验证通过后进入正在评审。",
      uiSummary: [
        "执行助手正在处理。",
        "还没有进入评审。",
      ],
    },
    in_review: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务代码和本地验证已经完成，正在等待 PR/MR 合并或最终核对。",
      ownerRoleLabel: roleLabel,
      stageAction: "整理交付结果，创建评审请求，等待自动合并或人工合并。",
      stageOutputs: [
        "交付材料应已生成。",
        "验证结果应已记录。",
        "等待合并凭证和最终写回。",
      ],
      nextEntry: "PR/MR 合并后写回已完成。",
      uiSummary: [
        "实现已完成。",
        "等待合并或最终写回。",
      ],
    },
    done: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务已经完成写回，执行链路到这里收口。",
      ownerRoleLabel: roleLabel,
      stageAction: "保留交付和证据，等待后续独立审计或查看交付。",
      stageOutputs: [
        "任务状态已写回 done。",
        "交付材料已保留。",
        "证据记录已保留。",
      ],
      nextEntry: "需要时查看交付，不自动进入审计。",
      uiSummary: [
        "任务已完成。",
        "主链路已经结束。",
      ],
    },
    blocked: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务被关键阻断项卡住，当前不能继续推进。",
      ownerRoleLabel: roleLabel,
      stageAction: "先解除阻断，再重新回到待执行阶段。",
      stageOutputs: [
        hasDependencies ? `阻断依赖：${task.dependencies.join("、")}` : "阻断原因：等待补充。",
        "当前不会继续执行。",
      ],
      nextEntry: "解除阻断后回到 backlog 或 todo。",
      uiSummary: [
        "任务被卡住。",
        "需要先处理阻断原因。",
      ],
    },
    cancel: {
      status,
      label: displayStatusLabelZh(status),
      businessMeaning: "任务已取消，不再继续推进。",
      ownerRoleLabel: roleLabel,
      stageAction: "保留任务记录，不再执行后续动作。",
      stageOutputs: [
        "当前任务已停止。",
        "不会再进入执行或评审。",
      ],
      nextEntry: "如需恢复，重新生成新任务。",
      uiSummary: [
        "任务已取消。",
        "当前链路终止。",
      ],
    },
  };

  return contracts[status];
}

function statusOwnerRoleLabel(status: IssueDisplayStatus, issueCategory?: IssueCategory) {
  if (issueCategory === "audit") {
    if (status === "done") {
      return "审计助手";
    }
    if (status === "blocked" || status === "cancel") {
      return "审计助手 / 人";
    }
    return "审计助手";
  }

  switch (status) {
    case "backlog":
      return "需求助手";
    case "todo":
    case "in_progress":
    case "in_review":
    case "done":
      return "执行助手";
    case "blocked":
    case "cancel":
      return "执行助手 / 人";
    default:
      return "执行助手";
  }
}

function inputIssueToTaskIssueNode(
  issue: InputIssue,
  indexed: IssueStatusIndexEntry | undefined,
  relationMap: Map<string, { blockedBy: Set<string>; blocks: Set<string> }>,
): TaskIssueNode {
  const issueCategory = issue.issueCategory ?? "spec";
  const displayStatus = indexed?.displayStatus ?? issue.displayStatus ?? displayStatusFromInputStatus(issue.status);
  const relation = relationMap.get(issue.issueId) ?? { blockedBy: new Set<string>(), blocks: new Set<string>() };
  return {
    auditStatus: indexed?.auditStatus ?? "not-requested",
    blockedBy: [...relation.blockedBy],
    blocks: [...relation.blocks],
    deliveryStatus: indexed?.deliveryStatus ?? "missing",
    displayStatus,
    evidenceStatus: indexed?.evidenceStatus ?? "missing",
    executeStatus: indexed?.executeStatus ?? null,
    expectedOutputs: issue.expectedOutputs ?? {},
    id: issue.issueId,
    issue,
    issueCategory,
    latestRunId: indexed?.latestRunId ?? null,
    projectId: issue.projectId ?? null,
    priority: indexed?.priority ?? issue.priority ?? "p2",
    requiredAgentRole: issue.requiredAgentRole ?? defaultAgentRoleForIssueCategory(issueCategory),
    executionRisk: indexed?.executionRisk ?? issue.executionRisk,
    sourceSpecId: issue.sourceSpecId ?? null,
    sourceSpecPath: issue.sourceSpecPath ?? null,
    status: issue.status,
    summary: issue.summary,
    title: issue.title,
    warnings: [],
  };
}

export function sortTasksByExecutionOrder<T extends ExecutionOrderedTask>(issues: T[]): T[] {
  const nodes = issues.map((issue, index) => ({
    dependents: [] as string[],
    externalDependencyCount: 0,
    id: issue.id,
    localDependencyCount: 0,
    remainingDependencyCount: 0,
    issue,
    position: index,
  }));
  const nodeById = new Map(nodes.map((node) => [node.id, node]));

  for (const node of nodes) {
    const dedupedDependencies = [...new Set(executionDependencies(node.issue))];
    const localDependencies = dedupedDependencies.filter((dependencyId) => nodeById.has(dependencyId));
    node.localDependencyCount = localDependencies.length;
    node.externalDependencyCount = dedupedDependencies.length - localDependencies.length;
    node.remainingDependencyCount = node.localDependencyCount + node.externalDependencyCount;

    for (const dependencyId of localDependencies) {
      nodeById.get(dependencyId)?.dependents.push(node.id);
    }
  }

  const sorted: T[] = [];
  const scheduled = new Set<string>();
  const ready = nodes.filter((node) => node.remainingDependencyCount === 0);
  ready.sort(compareExecutionCandidate);

  while (ready.length) {
    const current = ready.shift();
    if (!current || scheduled.has(current.id)) {
      continue;
    }

    scheduled.add(current.id);
    sorted.push(current.issue);

    for (const dependentId of current.dependents) {
      const dependent = nodeById.get(dependentId);
      if (!dependent || scheduled.has(dependent.id)) {
        continue;
      }
      dependent.remainingDependencyCount = Math.max(0, dependent.remainingDependencyCount - 1);
      if (dependent.remainingDependencyCount === 0) {
        ready.push(dependent);
      }
    }

    ready.sort(compareExecutionCandidate);
  }

  const pending = nodes
    .filter((node) => !scheduled.has(node.id))
    .sort(compareExecutionCandidate)
    .map((node) => node.issue);

  return [...sorted, ...pending];
}

function executionDependencies(issue: ExecutionOrderedTask) {
  const dependencies = issue.dependencies ?? issue.blockedBy ?? [];
  return dependencies.filter(Boolean);
}

function compareExecutionCandidate<T extends ExecutionOrderedTask>(
  left: {
    id: string;
    issue: T;
    localDependencyCount: number;
    position: number;
    remainingDependencyCount: number;
  },
  right: {
    id: string;
    issue: T;
    localDependencyCount: number;
    position: number;
    remainingDependencyCount: number;
  },
) {
  const remainingDiff = left.remainingDependencyCount - right.remainingDependencyCount;
  if (remainingDiff) {
    return remainingDiff;
  }

  const dependencyDiff = left.localDependencyCount - right.localDependencyCount;
  if (dependencyDiff) {
    return dependencyDiff;
  }

  const positionDiff = left.position - right.position;
  if (positionDiff) {
    return positionDiff;
  }

  const priorityDiff = priorityRank(left.issue.priority) - priorityRank(right.issue.priority);
  if (priorityDiff) {
    return priorityDiff;
  }

  const statusDiff = executionStatusRank(left.issue.displayStatus) - executionStatusRank(right.issue.displayStatus);
  if (statusDiff) {
    return statusDiff;
  }

  const timeDiff = executionSortTime(right.issue) - executionSortTime(left.issue);
  if (timeDiff) {
    return timeDiff;
  }

  return left.id.localeCompare(right.id);
}

function executionStatusRank(status?: IssueDisplayStatus | null) {
  const normalized = status ?? "backlog";
  return {
    in_progress: 0,
    todo: 1,
    in_review: 2,
    backlog: 3,
    blocked: 4,
    done: 5,
    cancel: 6,
  }[normalized] ?? 3;
}

function executionSortTime(issue: ExecutionOrderedTask) {
  return issue.updatedAt ?? issue.createdAt ?? 0;
}

function priorityRank(priority?: string | null) {
  const normalized = (priority ?? "p2").toLowerCase();
  return { p0: 0, p1: 1, p2: 2, p3: 3 }[normalized as "p0" | "p1" | "p2" | "p3"] ?? 2;
}

function buildTaskRelationMap(
  issues: InputIssue[],
  relations: InputIssueRelations | null | undefined,
  warnings: TaskProjectTreeWarning[],
) {
  const issueIds = new Set(issues.map((issue) => issue.issueId));
  const relationMap = new Map<string, { blockedBy: Set<string>; blocks: Set<string> }>();
  issueIds.forEach((issueId) => relationMap.set(issueId, { blockedBy: new Set<string>(), blocks: new Set<string>() }));

  issues.forEach((issue) => {
    issue.relations?.blockedBy?.forEach((blockedByIssueId) =>
      addTaskRelation(relationMap, issueIds, warnings, {
        fromIssueId: issue.issueId,
        toIssueId: blockedByIssueId,
        type: "blocked-by",
      }),
    );
    issue.relations?.blocks?.forEach((blockedIssueId) =>
      addTaskRelation(relationMap, issueIds, warnings, {
        fromIssueId: issue.issueId,
        toIssueId: blockedIssueId,
        type: "blocks",
      }),
    );
  });

  relationEdges(relations).forEach((relation) => addTaskRelation(relationMap, issueIds, warnings, relation));
  return relationMap;
}

function relationEdges(relations: InputIssueRelations | null | undefined): InputIssueRelationEdge[] {
  return relations?.relations ?? relations?.edges ?? [];
}

function addTaskRelation(
  relationMap: Map<string, { blockedBy: Set<string>; blocks: Set<string> }>,
  issueIds: Set<string>,
  warnings: TaskProjectTreeWarning[],
  relation: InputIssueRelationEdge,
) {
  const fromExists = issueIds.has(relation.fromIssueId);
  const toExists = issueIds.has(relation.toIssueId);
  if (!fromExists || !toExists) {
    warnings.push(
      taskProjectTreeWarning(
        "missing-relation-issue",
        `Relation ${relation.fromIssueId} -> ${relation.toIssueId} 指向缺失任务。`,
        undefined,
        fromExists ? relation.toIssueId : relation.fromIssueId,
      ),
    );
    return;
  }

  const from = relationMap.get(relation.fromIssueId);
  const to = relationMap.get(relation.toIssueId);
  if (!from || !to) {
    return;
  }

  if (relation.type === "blocks") {
    from.blocks.add(relation.toIssueId);
    to.blockedBy.add(relation.fromIssueId);
  }
  if (relation.type === "blocked-by") {
    from.blockedBy.add(relation.toIssueId);
    to.blocks.add(relation.fromIssueId);
  }
}

function projectIssueOrder(project: InputProject, issues: InputIssue[]) {
  const orderedIssueIds = new Set(project.issueIds);
  issues
    .filter((issue) => issue.projectId === project.projectId)
    .forEach((issue) => orderedIssueIds.add(issue.issueId));
  return [...orderedIssueIds];
}

function taskProjectTreeCounts(issues: TaskIssueNode[], projectCount: number): TaskProjectTreeCounts {
  return {
    activeIssueCount: issues.filter((issue) => issue.displayStatus === "in_progress").length,
    auditIssueCount: issues.filter((issue) => issue.issueCategory === "audit").length,
    doneIssueCount: issues.filter((issue) => issue.displayStatus === "done").length,
    issueCount: issues.length,
    projectCount,
  };
}

function pickTaskSelection(
  groups: TaskProjectGroup[],
  ungroupedIssues: TaskIssueNode[],
  activeIssueId?: string | null,
): TaskSelection {
  const issues = [...groups.flatMap((group) => group.issues), ...ungroupedIssues];
  const activeIssue = activeIssueId ? issues.find((issue) => issue.id === activeIssueId) : null;
  const issue =
    activeIssue ??
    issues.find((item) => item.displayStatus === "in_progress") ??
    issues.find((item) => item.displayStatus === "todo");
  if (issue) {
    return {
      issueId: issue.id,
      kind: "issue",
      projectId: issue.projectId ?? null,
    };
  }
  const project = groups.at(0);
  if (project) {
    return {
      kind: "project",
      projectId: project.id,
    };
  }
  return { kind: "empty" };
}

function defaultAgentRoleForIssueCategory(issueCategory: IssueCategory): AgentRole {
  return issueCategory === "audit" ? "audit-agent" : "build-agent";
}

function displayStatusFromInputStatus(status: InputIssueStatus): IssueDisplayStatus {
  return status;
}

function taskProjectTreeWarning(
  kind: TaskProjectTreeWarningKind,
  message: string,
  projectId?: string,
  issueId?: string,
): TaskProjectTreeWarning {
  return {
    issueId,
    kind,
    message,
    projectId,
  };
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
