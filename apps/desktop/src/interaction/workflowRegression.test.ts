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
  buildTaskExecutionProjection,
  buildTaskStatusContract,
  displayStatusLabelZh,
  taskActionsForTask,
} from "./viewModels";

type WorkflowStatus = Extract<IssueDisplayStatus, "todo" | "in_progress" | "in_review" | "done">;

const workflowStatuses: WorkflowStatus[] = ["todo", "in_progress", "in_review", "done"];

const statusExpectations: Record<
  WorkflowStatus,
  {
    action: string;
    nextEntry: string;
    output: string;
  }
> = {
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
    executeStatus: status === "in_progress" ? "running" : status === "todo" ? "planned" : "completed",
    expectedOutputs: {
      evidencePath: `.agentflow/output/evidence/run-${status}.json`,
      executeRunDir: `.agentflow/execute/runs/run-${status}`,
      releaseDeliveryDir: `.agentflow/output/release/run-${status}`,
      releaseNotePath: `.agentflow/output/release/run-${status}/release-note.md`,
    },
    executionPipeline: null,
    executionRisk: "medium",
    forbiddenActions: ["不创建审计"],
    forbiddenFiles: [".agentflow/output/audit/**"],
    goal: `验证 ${status} 状态`,
    id: `issue-${status}`,
    issueCategory: "spec",
    latestRunId: status === "todo" ? null : `run-${status}`,
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
  if (status === "todo") {
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
    auditPath: ".agentflow/output/audit/audit-workflow-regression/audit.json",
    reportPath: ".agentflow/output/audit/audit-workflow-regression/audit-report.md",
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

function assertWorkflowStatusContracts() {
  for (const status of workflowStatuses) {
    const task = workflowTask(status);
    const contract = buildTaskStatusContract(task);
    const expected = statusExpectations[status];

    assertEqual(contract.label, displayStatusLabelZh(status), `${status} label`);
    assertEqual(contract.ownerRoleLabel, "执行助手", `${status} owner`);
    assertEqual(contract.stageAction, expected.action, `${status} action`);
    assertIncludes(contract.stageOutputs, expected.output, `${status} output`);
    assertEqual(contract.nextEntry, expected.nextEntry, `${status} next entry`);
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
  assertIncludes(todoProjection.summaryItems, "交付包状态：未到生成阶段", "todo delivery state");

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
  assertIncludes(reviewProjection.summaryItems, "交付包状态：已生成草稿", "in_review delivery status");
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
  assertIncludes(doneProjection.summaryItems, "交付包状态：已交付", "done delivery status");
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

function assertBrowserPreviewWorkflowData() {
  const input = createBrowserPreviewInputSnapshot();
  const statusIndex = createBrowserPreviewIssueStatusIndex();
  const sessions = createBrowserPreviewMcpSessions();
  const output = createBrowserPreviewOutputIndex();
  const audit = createBrowserPreviewAuditIndex();
  const issues = new Map(input.issues.map((issue) => [issue.issueId, issue]));
  const statusByIssue = new Map(statusIndex.issues.map((issue) => [issue.issueId, issue]));

  assertEqual(issues.get("iss-ready")?.displayStatus, "todo", "preview todo issue");
  assertEqual(statusByIssue.get("iss-progress")?.latestRunId, "run-browser-preview-001", "preview in_progress run");
  assertEqual(statusByIssue.get("iss-review")?.latestRunId, "run-browser-preview-002", "preview in_review run");
  assertEqual(statusByIssue.get("iss-done")?.latestRunId, "run-browser-preview-003", "preview done run");
  assert(sessions.some((item) => item.issueId === "iss-review" && item.status === "in-review"), "preview review session");
  assert(sessions.some((item) => item.issueId === "iss-done" && item.mergeState === "merged"), "preview done session");
  assert(output.releaseDeliveries.some((item) => item.issueId === "iss-review" && item.runId === "run-browser-preview-002"), "preview review delivery");
  assert(output.releaseDeliveries.some((item) => item.issueId === "iss-done" && item.runId === "run-browser-preview-003"), "preview done delivery");
  assert(audit.audits.every((item) => item.sourceIssueId === "iss-done"), "preview audit stays after delivery");
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
  assertWorkflowActions();
  assertDeliveryProjection();
  assertExecutionProjection();
  assertBrowserPreviewWorkflowData();
}

runWorkflowRegressionChecks();
