import {
  createBrowserPreviewAuditIndex,
  createBrowserPreviewInputSnapshot,
  createBrowserPreviewIssueStatusIndex,
  createBrowserPreviewMcpSessions,
  createBrowserPreviewOutputIndex,
} from "../browserPreviewData";
import type { AuditIndexEntry, IssueDisplayStatus, McpSessionSnapshot, OutputIndexEntry, V1Issue } from "../types";
import {
  buildTaskDeliveryProjection,
  buildTaskCurrentStageSections,
  buildTaskExecutionProjection,
  buildTaskProjectTreeViewModel,
  buildTaskStatusContract,
  buildTaskStatusTimeline,
  buildTaskWorkflowYamlModel,
  displayStatusLabelZh,
  taskActionsForTask,
  type TaskIssueNode,
} from "./viewModels";

type WorkflowStatus = IssueDisplayStatus;

const workflowStatuses: WorkflowStatus[] = ["backlog", "todo", "in_progress", "in_review", "done", "blocked", "cancel"];

const statusExpectations: Record<
  WorkflowStatus,
  {
    action: string;
    nextEntry: string;
    output: string;
  }
> = {
  backlog: {
    action: "整理任务边界，确认范围、非目标和依赖关系。",
    nextEntry: "先确认任务合同，再进入执行前置检测。",
    output: "任务合同已生成。",
  },
  blocked: {
    action: "先解除阻断，再重新回到待执行阶段。",
    nextEntry: "解除阻断后回到 backlog 或 todo。",
    output: "当前不会继续执行。",
  },
  cancel: {
    action: "保留任务记录，不再执行后续动作。",
    nextEntry: "如需恢复，重新生成新任务。",
    output: "当前任务已停止。",
  },
  done: {
    action: "保留交付和证据，等待后续独立审计或查看交付。",
    nextEntry: "需要时查看交付，不自动进入审计。",
    output: "交付材料已保留。",
  },
  in_progress: {
    action: "完成测试设计、实现改动和沙箱验证。",
    nextEntry: "验证通过后进入正在评审。",
    output: "本地验证结果会在进入评审前补齐。",
  },
  in_review: {
    action: "整理交付结果，创建评审请求，等待自动合并或人工合并。",
    nextEntry: "PR/MR 合并后写回已完成。",
    output: "交付材料应已生成。",
  },
  todo: {
    action: "执行前置检测，确认合同完整、Context Pack 可读、工作区干净。",
    nextEntry: "通过前置检测后进入正在做。",
    output: "当前 run 已准备就绪。",
  },
};

function workflowTask(status: WorkflowStatus): V1Issue {
  const executionStarted = status === "in_progress" || status === "in_review" || status === "done";
  return {
    acceptanceCriteria: ["状态展示链路可验证。"],
    allowedFiles: ["apps/desktop/src/**"],
    auditStatus: status === "done" ? "passed" : status === "in_review" ? "not-requested" : "not-requested",
    boundary: ["不创建审计。"],
    codexInstructions: [],
    dependencies: [],
    deliveryStatus: status === "done" ? "delivered" : status === "in_review" ? "drafted" : "missing",
    displayStatus: status,
    evidenceRequired: ["本地验证证据。"],
    evidenceStatus: status === "done" || status === "in_review" ? "complete" : "missing",
    executeStatus:
      status === "in_progress"
        ? "running"
        : status === "todo"
          ? "planned"
          : status === "blocked"
            ? "blocked"
            : status === "cancel"
              ? "cancelled"
              : "completed",
    expectedOutputs: {
      evidencePath: `.agentflow/tasks/issue-${status}/evidence/evidence.json`,
      executeRunDir: `.agentflow/execute/runs/run-${status}`,
      releaseDeliveryDir: `.agentflow/output/release/run-${status}`,
      releaseNotePath: `.agentflow/output/release/run-${status}/release-note.md`,
    },
    executionPipeline: null,
    executionRisk: "medium",
    forbiddenActions: ["不创建审计"],
    forbiddenFiles: [".agentflow/audit/**"],
    goal: `验证 ${status} 状态`,
    id: `issue-${status}`,
    issueCategory: "spec",
    latestRunId: executionStarted ? `run-${status}` : null,
    nonGoals: ["不扩展新功能。"],
    priority: "p1",
    rawStatus: status,
    requiredAgentRole: "build-agent",
    scope: ["验证任务状态、执行摘要、交付摘要同步。"],
    status,
    title: `状态链路 ${status}`,
    validationCommands: ["npm --prefix apps/desktop run build", "git diff --check"],
  };
}

function outputEntry(runId: string, issueId: string, status: string): OutputIndexEntry {
  return {
    issueId,
    path: `.agentflow/output/release/${runId}/delivery.json`,
    runId,
    sourceSpecId: "spec-workflow-regression",
    status,
    updatedAt: 1780291200,
  };
}

function session(status: WorkflowStatus): McpSessionSnapshot | null {
  if (status === "backlog" || status === "todo" || status === "blocked" || status === "cancel") {
    return null;
  }
  return {
    branchName: `agentflow/workflow/${status}`,
    createdAt: 1780291000,
    issueId: `issue-${status}`,
    launchMode: "cli-exec-stdin",
    launchRequestPath: `.agentflow/execute/runs/run-${status}/launcher/build-agent-request.json`,
    logPath: `.agentflow/state/mcp/sessions/codex-run-${status}.jsonl`,
    mergeState: status === "done" ? "merged" : status === "in_review" ? "open" : null,
    note: null,
    pid: null,
    planPath: `.agentflow/state/mcp/plans/codex-run-${status}.json`,
    prUrl: status === "in_review" || status === "done" ? `https://example.invalid/pr/${status}` : null,
    provider: "codex",
    remoteSessionId: null,
    runId: `run-${status}`,
    sessionId: `codex-run-${status}`,
    status: status === "done" ? "done" : status === "in_review" ? "in-review" : "running",
    updatedAt: 1780291200,
    version: "agentflow-mcp-session.workflow-regression",
  };
}

function auditForDone(): AuditIndexEntry {
  return {
    auditId: "audit-workflow-regression",
    auditPath: ".agentflow/audit/audit-workflow-regression/audit.json",
    reportPath: ".agentflow/audit/audit-workflow-regression/audit-report.md",
    requestedAt: 1780291200,
    requestedBy: "workflow-regression",
    sourceDeliveryId: "run-done",
    sourceIssueId: "issue-done",
    sourceRunId: "run-done",
    sourceSpecId: "spec-workflow-regression",
    status: "passed-with-warnings",
    trigger: "release-auto",
  };
}

function taskFromPreviewNode(node: TaskIssueNode): V1Issue {
  return {
    acceptanceCriteria: node.issue.acceptanceCriteria,
    allowedFiles: node.issue.allowedPaths?.length ? node.issue.allowedPaths : node.issue.scope,
    auditStatus: node.auditStatus,
    boundary: node.issue.nonGoals,
    codexInstructions: node.issue.validationHints,
    dependencies: node.blockedBy,
    deliveryStatus: node.deliveryStatus,
    displayStatus: node.displayStatus,
    evidenceRequired: node.issue.acceptanceCriteria,
    evidenceStatus: node.evidenceStatus,
    executeStatus: node.executeStatus ?? null,
    expectedOutputs: node.expectedOutputs,
    executionPipeline: node.issue.executionPipeline ?? null,
    executionRisk: node.executionRisk,
    forbiddenActions: node.issue.forbiddenActions ?? [],
    forbiddenFiles: node.issue.forbiddenPaths ?? [],
    goal: node.summary,
    id: node.id,
    issueCategory: node.issueCategory,
    latestRunId: node.latestRunId ?? null,
    nonGoals: node.issue.nonGoals,
    priority: node.priority,
    rawStatus: node.status,
    requiredAgentRole: node.requiredAgentRole,
    scope: node.issue.scope,
    sourceSpecId: node.sourceSpecId ?? null,
    sourceSpecPath: node.sourceSpecPath ?? null,
    status: node.status,
    title: node.title,
    validationCommands: node.issue.validationCommands?.length ? node.issue.validationCommands : node.issue.validationHints,
  };
}

function assertWorkflowStatusContracts() {
  const ownerByStatus: Record<WorkflowStatus, string> = {
    backlog: "需求助手",
    blocked: "执行助手 / 人",
    cancel: "执行助手 / 人",
    done: "执行助手",
    in_progress: "执行助手",
    in_review: "执行助手",
    todo: "执行助手",
  };

  for (const status of workflowStatuses) {
    const task = workflowTask(status);
    const contract = buildTaskStatusContract(task);
    const expected = statusExpectations[status];

    assertEqual(contract.label, displayStatusLabelZh(status), `${status} label`);
    assertEqual(contract.ownerRoleLabel, ownerByStatus[status], `${status} owner`);
    assertEqual(contract.stageAction, expected.action, `${status} action`);
    assertIncludes(contract.stageOutputs, expected.output, `${status} output`);
    assertEqual(contract.nextEntry, expected.nextEntry, `${status} next entry`);
  }
}

function assertWorkflowTimelineAndStageDetails() {
  const timelineCurrentByStatus: Record<WorkflowStatus, IssueDisplayStatus> = {
    backlog: "backlog",
    blocked: "blocked",
    cancel: "cancel",
    done: "done",
    in_progress: "in_progress",
    in_review: "in_review",
    todo: "todo",
  };
  const firstSectionByStatus: Record<WorkflowStatus, string> = {
    backlog: "当前阶段",
    blocked: "阻断信息",
    cancel: "取消信息",
    done: "最终结果",
    in_progress: "执行信息",
    in_review: "当前阶段",
    todo: "前置检测",
  };

  for (const status of workflowStatuses) {
    const task = workflowTask(status);
    const contract = buildTaskStatusContract(task);
    const timeline = buildTaskStatusTimeline(status, contract);
    const currentStep = timeline.find((step) => step.state === "current" || step.state === "exception");
    const sections = buildTaskCurrentStageSections({
      contract,
      executeItems: ["Run：测试运行。"],
      reviewItems: ["评审链接：测试链接。"],
      stageItems: contract.stageOutputs,
      status,
    });

    assertEqual(currentStep?.id, timelineCurrentByStatus[status], `${status} timeline current step`);
    assertEqual(sections.length, 3, `${status} current stage section count`);
    assertEqual(sections[0]?.title, firstSectionByStatus[status], `${status} current stage primary section`);
  }
}

function assertWorkflowActions() {
  assertIncludes(taskActionsForTask(workflowTask("todo")), "copy-handoff", "todo action");
  assertIncludes(taskActionsForTask(workflowTask("in_progress")), "check-writeback", "in_progress action");
  assertIncludes(taskActionsForTask(workflowTask("in_review")), "view-delivery", "in_review action");
  assertIncludes(taskActionsForTask(workflowTask("done")), "view-delivery", "done action");
}

function assertDeliveryProjection() {
  const todoProjection = buildTaskDeliveryProjection({
    audit: null,
    delivery: null,
    evidence: null,
    session: null,
    task: workflowTask("todo"),
  });
  assertEqual(todoProjection.missingItems.length, 0, "todo delivery missing items");
  assertIncludes(todoProjection.summaryItems, "交付包：未到生成阶段", "todo delivery state");

  const reviewTask = workflowTask("in_review");
  const reviewDelivery = outputEntry("run-in_review", reviewTask.id, "drafted");
  const reviewProjection = buildTaskDeliveryProjection({
    audit: null,
    delivery: reviewDelivery,
    evidence: outputEntry("run-in_review", reviewTask.id, "complete"),
    session: session("in_review"),
    task: reviewTask,
  });
  assertEqual(reviewProjection.missingItems.length, 0, "in_review delivery missing items");
  assertIncludes(reviewProjection.summaryItems, "交付包：run-in_review · 已生成草稿", "in_review delivery status");
  assertIncludes(reviewProjection.packageItems, "交付说明：.agentflow/output/release/run-in_review/release-note.md", "in_review release note path");
  assertIncludes(reviewProjection.summaryItems, "审计提示：交付后的独立入口。", "in_review audit independence");

  const doneTask = workflowTask("done");
  const doneProjection = buildTaskDeliveryProjection({
    audit: auditForDone(),
    delivery: outputEntry("run-done", doneTask.id, "delivered"),
    evidence: outputEntry("run-done", doneTask.id, "complete"),
    session: session("done"),
    task: doneTask,
  });
  assertEqual(doneProjection.deliveryRunId, "run-done", "done delivery run");
  assertEqual(doneProjection.missingItems.length, 0, "done delivery missing items");
  assertIncludes(doneProjection.summaryItems, "交付包：run-done · 已交付", "done delivery status");
  assertIncludes(doneProjection.packageItems, "交付说明：.agentflow/output/release/run-done/release-note.md", "done release note path");
  assertIncludes(doneProjection.packageItems, "后续审计：通过，有警告", "done audit remains projection only");
}

function assertExecutionProjection() {
  const progressTask = workflowTask("in_progress");
  const progressProjection = buildTaskExecutionProjection({
    executeWorkspaceStatus: "ready",
    mcpSessionsSource: "tauri",
    session: session("in_progress"),
    task: progressTask,
  });
  assertEqual(progressProjection.runId, "run-in_progress", "in_progress execution run");
  assertEqual(progressProjection.missingItems.length, 0, "in_progress execution missing items");
  assertIncludes(progressProjection.summaryItems, "Execute status：正在执行", "in_progress execute status");
  assertIncludes(progressProjection.summaryItems, "Session：运行中", "in_progress session status");
  assertIncludes(progressProjection.validationItems, "Validation：2 条验证命令", "in_progress validation summary");

  const missingProjection = buildTaskExecutionProjection({
    executeWorkspaceStatus: "ready",
    mcpSessionsSource: "tauri",
    session: null,
    task: {
      ...progressTask,
      latestRunId: null,
      validationCommands: [],
    },
  });
  assertIncludes(missingProjection.missingItems, "Run：当前状态需要 run，但任务索引未记录。", "missing run");
  assertIncludes(missingProjection.missingItems, "Session：当前状态通常应有会话记录，当前未读取到。", "missing session");
  assertIncludes(missingProjection.missingItems, "Validation：未登记验证命令。", "missing validation");
}

function assertWorkflowYamlProjection() {
  const progressTask = workflowTask("in_progress");
  const progressYaml = buildTaskWorkflowYamlModel({
    contract: buildTaskStatusContract(progressTask),
    deliveryProjection: buildTaskDeliveryProjection({
      audit: null,
      delivery: null,
      evidence: null,
      session: session("in_progress"),
      task: progressTask,
    }),
    executionProjection: buildTaskExecutionProjection({
      executeWorkspaceStatus: "ready",
      mcpSessionsSource: "tauri",
      session: session("in_progress"),
      task: progressTask,
    }),
    task: progressTask,
  });
  assertEqual(progressYaml.fileName, "workflow.yml", "yaml file name");
  assertIncludes(progressYaml.content.split("\n"), "task:", "yaml task root");
  assert(progressYaml.content.includes("workflow:"), "yaml workflow root");
  assert(progressYaml.content.includes("execution:"), "yaml execution root");
  assert(progressYaml.content.includes("delivery:"), "yaml delivery root");
  assert(progressYaml.content.includes("result:"), "yaml result root");
  assert(progressYaml.content.includes('  id: "issue-in_progress"'), "yaml task id");
  assert(progressYaml.content.includes('  executeStatus: "running"'), "yaml execute status");
  assert(progressYaml.content.includes('  evidenceStatus: "missing"'), "yaml evidence status");
  assert(progressYaml.content.includes('  deliveryStatus: "missing"'), "yaml delivery status");
  assert(progressYaml.content.includes('  finalState: "not_final"'), "yaml result final state");

  const doneTask = workflowTask("done");
  const doneYaml = buildTaskWorkflowYamlModel({
    contract: buildTaskStatusContract(doneTask),
    deliveryProjection: buildTaskDeliveryProjection({
      audit: auditForDone(),
      delivery: outputEntry("run-done", doneTask.id, "delivered"),
      evidence: outputEntry("run-done", doneTask.id, "complete"),
      session: session("done"),
      task: doneTask,
    }),
    executionProjection: buildTaskExecutionProjection({
      executeWorkspaceStatus: "ready",
      mcpSessionsSource: "tauri",
      session: session("done"),
      task: doneTask,
    }),
    task: doneTask,
  });
  assert(doneYaml.content.includes('  id: "issue-done"'), "yaml switched task id");
  assert(doneYaml.content.includes('  finalState: "delivered"'), "yaml done result final state");
  assert(progressYaml.content !== doneYaml.content, "yaml content changes by task");
}

function assertBrowserPreviewWorkflowData() {
  const input = createBrowserPreviewInputSnapshot();
  const statusIndex = createBrowserPreviewIssueStatusIndex();
  const sessions = createBrowserPreviewMcpSessions();
  const output = createBrowserPreviewOutputIndex();
  const audit = createBrowserPreviewAuditIndex();
  const issues = new Map(input.issues.map((issue) => [issue.issueId, issue]));
  const statusByIssue = new Map(statusIndex.issues.map((issue) => [issue.issueId, issue]));

  assertEqual(issues.get("iss-ready")?.displayStatus, "todo", "preview todo issue");
  assertEqual(issues.get("iss-review")?.title, "整理评审交付", "preview review title");
  assertEqual(issues.get("iss-review")?.summary, "执行和本地验证已完成，等待 PR/MR 合并。", "preview review summary");
  assertEqual(issues.get("iss-done")?.summary, "交付已写回，后续审计保持独立入口。", "preview done summary");
  assertEqual(statusByIssue.get("iss-progress")?.latestRunId, "run-browser-preview-001", "preview in_progress run");
  assertEqual(statusByIssue.get("iss-review")?.latestRunId, "run-browser-preview-002", "preview in_review run");
  assertEqual(statusByIssue.get("iss-done")?.latestRunId, "run-browser-preview-003", "preview done run");
  assertEqual(statusByIssue.get("iss-review")?.auditStatus, "not-requested", "preview review audit stays independent");
  assert(sessions.some((item) => item.issueId === "iss-review" && item.status === "in-review"), "preview review session");
  assert(sessions.some((item) => item.issueId === "iss-done" && item.mergeState === "merged"), "preview done session");
  assert(output.releaseDeliveries.some((item) => item.issueId === "iss-review" && item.runId === "run-browser-preview-002"), "preview review delivery");
  assert(output.releaseDeliveries.some((item) => item.issueId === "iss-done" && item.runId === "run-browser-preview-003"), "preview done delivery");
  assert(audit.audits.every((item) => item.trigger === "human-via-agent"), "preview audit trigger stays independent");
  assert(audit.audits.every((item) => item.sourceIssueId === "iss-done"), "preview audit stays after delivery");
  assert(issues.get("iss-progress")?.validationCommands?.includes("npm --prefix apps/desktop run build"), "preview build validation command");
  assert(issues.get("iss-progress")?.validationCommands?.includes("git diff --check"), "preview diff validation command");
  assertEqual(issues.get("iss-progress")?.validationCommands?.includes("cargo test"), false, "preview avoids backend validation command");
}

function assertBrowserPreviewTaskWorkspaceSmoke() {
  const input = createBrowserPreviewInputSnapshot();
  const statusIndex = createBrowserPreviewIssueStatusIndex();
  const sessions = createBrowserPreviewMcpSessions();
  const output = createBrowserPreviewOutputIndex();
  const audit = createBrowserPreviewAuditIndex();
  const tree = buildTaskProjectTreeViewModel({
    activeIssueId: "iss-progress",
    issueStatusIndex: statusIndex,
    issues: input.issues,
    projects: input.projects,
    relations: input.relations,
  });
  const project = tree.groups.find((group) => group.id === "project-browser-preview");
  assert(project, "preview project group");
  assertEqual(tree.selection.kind, "issue", "preview task selection kind");
  assertEqual(tree.selection.kind === "issue" ? tree.selection.issueId : null, "iss-progress", "preview active task selection");
  assertEqual(project.counts.activeIssueCount, 1, "preview active issue count");
  assertEqual(project.counts.doneIssueCount, 1, "preview done issue count");

  for (const issueId of ["iss-progress", "iss-review", "iss-done"]) {
    const node = project.issues.find((item) => item.id === issueId);
    assert(node, `${issueId} node`);
    const task = taskFromPreviewNode(node);
    const session = sessions.find((item) => item.issueId === issueId) ?? null;
    const delivery = output.releaseDeliveries.find((item) => item.issueId === issueId) ?? null;
    const evidence = output.evidence.find((item) => item.issueId === issueId) ?? null;
    const linkedAudit = delivery
      ? audit.audits.find((item) => item.sourceIssueId === issueId || item.sourceRunId === delivery.runId) ?? null
      : null;
    const contract = buildTaskStatusContract(task);
    const executionProjection = buildTaskExecutionProjection({
      executeWorkspaceStatus: "ready",
      mcpSessionsSource: "preview",
      session,
      task,
    });
    const deliveryProjection = buildTaskDeliveryProjection({
      audit: linkedAudit,
      delivery,
      evidence,
      session,
      task,
    });
    const yaml = buildTaskWorkflowYamlModel({
      contract,
      deliveryProjection,
      executionProjection,
      task,
    });

    assertEqual(executionProjection.missingItems.length, 0, `${issueId} execution completeness`);
    assertEqual(deliveryProjection.missingItems.length, 0, `${issueId} delivery completeness`);
    assert(yaml.content.includes("workflow:"), `${issueId} yaml workflow panel`);
    assert(yaml.content.includes("execution:"), `${issueId} yaml execution panel`);
    assert(yaml.content.includes("delivery:"), `${issueId} yaml delivery panel`);
    assert(yaml.content.includes(`  id: "${issueId}"`), `${issueId} yaml selected task`);
  }
}

function assert(condition: unknown, label: string): asserts condition {
  if (!condition) {
    throw new Error(`workflow regression failed: ${label}`);
  }
}

function assertEqual<T>(actual: T, expected: T, label: string) {
  assert(Object.is(actual, expected), `${label}: expected ${String(expected)}, got ${String(actual)}`);
}

function assertIncludes<T>(items: T[], expected: T, label: string) {
  assert(items.includes(expected), `${label}: missing ${String(expected)}`);
}

export function runWorkflowRegressionChecks() {
  assertWorkflowStatusContracts();
  assertWorkflowTimelineAndStageDetails();
  assertWorkflowActions();
  assertDeliveryProjection();
  assertExecutionProjection();
  assertWorkflowYamlProjection();
  assertBrowserPreviewWorkflowData();
  assertBrowserPreviewTaskWorkspaceSmoke();
}

runWorkflowRegressionChecks();
