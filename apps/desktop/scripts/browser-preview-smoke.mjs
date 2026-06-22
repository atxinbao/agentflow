import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");
const smokeRoot = mkdtempSync(path.join(tmpdir(), "agentflow-browser-preview-smoke-"));
const server = await createServer({
  root: desktopRoot,
  configFile: path.join(desktopRoot, "vite.config.ts"),
  logLevel: "error",
  server: { middlewareMode: true },
  appType: "custom",
});

try {
  const preview = await server.ssrLoadModule("/src/browserPreviewData.ts");
  const viewModels = await server.ssrLoadModule("/src/interaction/viewModels.ts");
  const projectRegistryModule = await server.ssrLoadModule("/src/projectRegistry.ts");
  const inputSnapshot = preview.createBrowserPreviewInputSnapshot(smokeRoot);
  const issueStatusIndex = preview.createBrowserPreviewIssueStatusIndex(smokeRoot);
  const outputIndex = preview.createBrowserPreviewOutputIndex();
  const auditIndex = preview.createBrowserPreviewAuditIndex();
  const auditReport = preview.createBrowserPreviewHumanAuditReport();
  const agentEnvironment = preview.createBrowserPreviewAgentEnvironmentStatus(smokeRoot);
  const stateStatus = preview.createBrowserPreviewStateStatus(smokeRoot);
  const projectRegistry = projectRegistryModule.createBrowserPreviewProjectRegistry(smokeRoot);
  const registryWithoutInactive = projectRegistryModule.removeProject(
    projectRegistry,
    "/Users/mac/Documents/mobile-app",
  );
  const registryAfterActiveRemoval = projectRegistryModule.removeProject(
    registryWithoutInactive,
    "/Users/mac/Documents/my-web-app",
  );
  const emptyRegistry = projectRegistryModule.removeProject(registryAfterActiveRemoval, smokeRoot);
  const storage = new Map();
  globalThis.window = {
    localStorage: {
      getItem: (key) => (storage.has(key) ? storage.get(key) : null),
      removeItem: (key) => {
        storage.delete(key);
      },
      setItem: (key, value) => {
        storage.set(key, String(value));
      },
    },
  };
  projectRegistryModule.persistProjectRegistry(emptyRegistry);
  const restoredEmptyRegistry = projectRegistryModule.readProjectRegistry({
    legacyActivePage: "files",
    legacyProjectRoot: "/Users/mac/Documents/legacy-project",
    projectNameFromRoot: () => "legacy-project",
  });
  const appEntry = readFileSync(path.join(desktopRoot, "src/App.tsx"), "utf8");
  const appShellCss = readFileSync(path.join(desktopRoot, "src/AppShell.css"), "utf8");
  const stateStatusHook = readFileSync(
    path.join(desktopRoot, "src/features/state/hooks/useStateStatus.ts"),
    "utf8",
  );
  const runtimeApiMapping = readFileSync(path.join(desktopRoot, "../../crates/runtime-api/src/mapping.rs"), "utf8");
  const runtimeApiCommands = readFileSync(path.join(desktopRoot, "../../crates/runtime-api/src/commands.rs"), "utf8");
  const projectLocalFilesPage = readFileSync(
    path.join(desktopRoot, "src/features/project-files/ProjectLocalFilesPage.tsx"),
    "utf8",
  );
  const designSystemPreview = readFileSync(
    path.join(desktopRoot, "src/features/design-system/DesignSystemPreview.tsx"),
    "utf8",
  );
  const designSystemFiles = [
    ["button", "src/components/Button.tsx"],
    ["surface-card", "src/components/SurfaceCard.tsx"],
    ["status-chip", "src/components/StatusChip.tsx"],
    ["metric-card", "src/components/MetricCard.tsx"],
    ["empty-state", "src/components/EmptyState.tsx"],
    ["blocked-state", "src/components/BlockedState.tsx"],
    ["loading-state", "src/components/LoadingState.tsx"],
    ["warning-state", "src/components/WarningState.tsx"],
    ["copyable-code-block", "src/components/CopyableCodeBlock.tsx"],
    ["advanced-details-drawer", "src/components/AdvancedDetailsDrawer.tsx"],
  ];
  assert.equal(inputSnapshot.issues.length, 7);
  assert.deepEqual(
    inputSnapshot.issues.map((issue) => issue.displayStatus),
    ["backlog", "todo", "in_progress", "in_review", "todo", "done", "cancel"],
  );
  assert.deepEqual(
    inputSnapshot.issues.map((issue) => issue.priority),
    ["p3", "p1", "p0", "p2", "p2", "p2", "p2"],
  );
  const taskProjections = inputSnapshot.issues
    .map((issue) => preview.createBrowserPreviewTaskProjection(issue.issueId, smokeRoot, "default"))
    .filter(Boolean);
  const todoProjection = preview.createBrowserPreviewTaskProjection("iss-ready", smokeRoot, "default");
  const progressProjection = preview.createBrowserPreviewTaskProjection("iss-progress", smokeRoot, "default");
  const reviewProjection = preview.createBrowserPreviewTaskProjection("iss-review", smokeRoot, "default");
  const auditQueuedProjection = preview.createBrowserPreviewTaskProjection("iss-audit-ready", smokeRoot, "default");
  const doneProjection = preview.createBrowserPreviewTaskProjection("iss-done", smokeRoot, "default");
  const projectProjection = preview.createBrowserPreviewProjectProjection("project-browser-preview", smokeRoot, "default");
  const specWorkbenchProjection = preview.createBrowserPreviewSpecWorkbenchProjection();
  const taskTree = viewModels.buildTaskProjectTreeViewModel({
    activeIssueId: "iss-progress",
    issues: inputSnapshot.issues,
    issueStatusIndex,
    projects: inputSnapshot.projects,
    relations: inputSnapshot.relations,
  });
  assert.equal(inputSnapshot.issues.some((issue) => "riskLevel" in issue), false);
  assert.equal(inputSnapshot.issues.every((issue) => "executionRisk" in issue), true);
  assert.deepEqual(
    issueStatusIndex.issues.map((issue) => issue.displayStatus),
    ["backlog", "todo", "in_progress", "in_review", "todo", "done", "cancel"],
  );
  assert.equal(outputIndex.evidence.length, 2);
  assert.equal(outputIndex.evidence[0].path, ".agentflow/tasks/iss-review/evidence/evidence.json");
  assert.equal(outputIndex.evidence[1].path, ".agentflow/tasks/iss-done/evidence/evidence.json");
  assert.equal(taskProjections.length, 7);
  assert.equal(taskTree.groups.length, 1);
  assert.equal(taskTree.groups[0].title, "浏览器预览任务项目");
  assert.equal(taskTree.groups[0].issues.length, 6);
  assert.equal(taskTree.ungroupedIssues.length, 1);
  assert.equal(taskTree.ungroupedIssues[0].id, "iss-backlog");
  assert.equal(taskTree.selection?.kind, "issue");
  assert.equal(taskTree.selection?.issueId, "iss-progress");
  assert.equal(todoProjection?.timeline.find((item) => item.state === "todo")?.phase, "current");
  assert.equal(todoProjection?.timeline.find((item) => item.state === "in_progress")?.phase, "future");
  assert.equal(todoProjection?.timeline.find((item) => item.state === "in_progress")?.events.length, 0);
  assert.equal(progressProjection?.timeline.find((item) => item.state === "in_progress")?.phase, "current");
  assert.equal(progressProjection?.timeline.find((item) => item.state === "in_progress")?.events.length, 3);
  assert.equal(progressProjection?.audit.status, "audit-running");
  assert.equal(progressProjection?.timeline.find((item) => item.state === "in_review")?.phase, "future");
  assert.equal(progressProjection?.timeline.find((item) => item.state === "in_review")?.events.length, 0);
  assert.equal(reviewProjection?.timeline.find((item) => item.state === "in_review")?.phase, "current");
  assert.equal(reviewProjection?.timeline.find((item) => item.state === "done")?.phase, "future");
  assert.equal(reviewProjection?.timeline.find((item) => item.state === "done")?.events.length, 0);
  assert.equal(auditQueuedProjection?.audit.status, "audit-requested");
  assert.equal(doneProjection?.timeline.find((item) => item.state === "done")?.phase, "current");
  assert.equal(doneProjection?.timeline.find((item) => item.state === "done")?.events.length, 2);
  assert.equal(doneProjection?.timeline.find((item) => item.state === "in_review")?.phase, "past");
  assert.equal(doneProjection?.audit.status, "audit-completed");
  assert.equal(progressProjection?.acceptance?.outcome, "pending");
  assert.equal(reviewProjection?.acceptance?.outcome, "needs-human-decision");
  assert.equal(doneProjection?.acceptance?.passed, true);
  assert.equal(doneProjection?.acceptance?.traceability.closeoutProofPath.includes("completion/commit.json"), true);
  assert.equal(projectProjection?.currentIssueId, "iss-progress");
  assert.equal(projectProjection?.stageLabel, "正在推进");
  assert.equal(projectProjection?.nextActionLabel, "继续当前任务");
  assert.equal(projectProjection?.completion?.currentState, "continue");
  assert.equal(projectProjection?.completion?.latestOutcome, "continue");
  assert.equal(projectProjection?.completion?.nextRecommendedActionLabel, "继续项目循环");
  assert.equal(
    projectProjection?.completionHint,
    "当前还有 4 条任务未完成，先继续推进任务循环。 最近交付：项目公开交付仍在围绕 iss-progress 整理。",
  );
  assert.equal(projectProjection?.blockers.length, 0);
  assert.equal(projectProjection?.projectBrain.nextRecommendedActionLabel, "进入项目循环");
  assert.equal(projectProjection?.delivery?.currentIssueId, "iss-progress");
  assert.equal(specWorkbenchProjection.requirements.length, 1);
  assert.equal(specWorkbenchProjection.specLoop?.stages.length, 8);
  assert.deepEqual(
    specWorkbenchProjection.specLoop?.authorityLayers.map((entry) => entry.authorityLayer),
    ["preview-artifact", "project-authority", "issue-authority"],
  );
  assert.equal(specWorkbenchProjection.preview?.issuePreview.length, 3);
  assert.equal(specWorkbenchProjection.specLoop?.runtimeActionProposals.length, 1);
  assert.equal(
    taskProjections.some((task) => task.publicDelivery?.prUrl?.includes("/pull/100")),
    true,
  );
  assert.equal(
    taskProjections.every((task) => !JSON.stringify(task.publicDelivery).includes(".agentflow/output/release")),
    true,
  );
  assert.equal(auditIndex.audits.length, 1);
  assert.equal(auditIndex.audits[0].auditId, "audit-browser-preview-001");
  assert.ok(auditReport.reportMarkdown.includes("Human Audit Browser Preview"));
  assert.equal(agentEnvironment.locale.manualLanguage, "en");
  assert.equal(agentEnvironment.locale.source, "browser-preview");
  assert.equal(agentEnvironment.style.styleId, "plain-work-style");
  assert.equal(agentEnvironment.style.appliesToCodeComments, true);
  assert.equal(agentEnvironment.skillsLock.skillCount, 7);
  assert.equal(
    agentEnvironment.skills.some((skill) => skill.name === "plain-work-style" && skill.hashMatches),
    true,
  );
  assert.equal(stateStatus.currentStage, "workspace-ready");
  assert.equal(stateStatus.auditStatus, "passed-with-warnings");
  assert.equal(projectRegistry.projects.length, 3);
  assert.deepEqual(
    projectRegistry.projects.map((project) => project.name),
    ["my-web-app", "AgentFlow", "mobile-app"],
  );
  assert.equal(projectRegistry.activeProjectRoot, "/Users/mac/Documents/my-web-app");
  assert.deepEqual([...projectRegistry.expandedProjectRoots], ["/Users/mac/Documents/my-web-app"]);
  assert.equal(projectRegistry.activePageByProject["/Users/mac/Documents/my-web-app"], "home");
  assert.equal(projectRegistry.activePageByProject[smokeRoot], "tasks");
  assert.equal(projectRegistry.activePageByProject["/Users/mac/Documents/mobile-app"], "files");
  assert.equal(projectRegistry.projects[2].status, "missing");
  assert.deepEqual(
    registryWithoutInactive.projects.map((project) => project.name),
    ["my-web-app", "AgentFlow"],
  );
  assert.equal(registryWithoutInactive.activeProjectRoot, "/Users/mac/Documents/my-web-app");
  assert.equal(registryAfterActiveRemoval.activeProjectRoot, smokeRoot);
  assert.equal(registryAfterActiveRemoval.expandedProjectRoots.has(smokeRoot), true);
  assert.deepEqual(emptyRegistry.projects, []);
  assert.equal(emptyRegistry.activeProjectRoot, null);
  assert.deepEqual([...emptyRegistry.expandedProjectRoots], []);
  assert.deepEqual(emptyRegistry.activePageByProject, {});
  assert.equal(storage.get(projectRegistryModule.projectRegistryStorageKeys.projects), "[]");
  assert.equal(storage.has(projectRegistryModule.projectRegistryStorageKeys.activeProjectRoot), false);
  assert.equal(storage.get(projectRegistryModule.projectRegistryStorageKeys.expandedProjectRoots), "[]");
  assert.equal(storage.get(projectRegistryModule.projectRegistryStorageKeys.activePageByProject), "{}");
  assert.deepEqual(restoredEmptyRegistry.projects, []);
  assert.equal(restoredEmptyRegistry.activeProjectRoot, null);
  assert.ok(appEntry.includes("createBrowserPreviewProjectRegistry"));
  assert.ok(appEntry.includes("readProjectRegistry"));
  assert.ok(appEntry.includes("persistProjectRegistry"));
  assert.ok(appEntry.includes("data-agentflow-project-select"));
  assert.ok(appEntry.includes("data-agentflow-project-toggle"));
  assert.ok(appEntry.includes("data-agentflow-project-remove"));
  assert.ok(appEntry.includes("data-agentflow-project-remove-confirm"));
  assert.ok(appEntry.includes("这只会把项目从 AgentFlow 侧边栏移除，不会删除你的本地文件。"));
  assert.ok(appEntry.includes("未选择项目 · 本地模式"));
  assert.ok(appEntry.includes("添加本地项目"));
  assert.ok(appEntry.includes("removeItem(interactionStorageKeys.projectRoot)"));
  assert.ok(appEntry.includes("data-agentflow-page-id"));
  assert.ok(appEntry.includes("agentManualState.status?.locale.agentLocale"));
  assert.ok(appEntry.includes("function buildAgentPullRequestTemplateZh"));
  assert.ok(appEntry.includes("## 大白话说明"));
  assert.ok(appEntry.includes("buildAgentPullRequestTemplate(task, agentLocale)"));
  assert.ok(appEntry.includes('data-agentflow-page="project-unavailable"'));
  assert.ok(appEntry.includes('data-agentflow-ux="v16"'));
  assert.ok(appEntry.includes('data-agentflow-screen="login"'));
  assert.ok(appEntry.includes('data-agentflow-screen="first-run"'));
  assert.ok(appEntry.includes('data-agentflow-page="workbench"'));
  assert.ok(appEntry.includes('data-agentflow-page="spec"'));
  assert.ok(appEntry.includes('data-agentflow-page="tasks"'));
  assert.ok(appEntry.includes('data-agentflow-page="files"'));
  assert.ok(appEntry.includes('data-agentflow-page="delivery"'));
  assert.ok(appEntry.includes('data-agentflow-page="audit"'));
  assert.ok(appEntry.includes('data-agentflow-page="advanced"'));
  const titleBarSource = appEntry.slice(
    appEntry.indexOf("function TitleBar"),
    appEntry.indexOf("function WindowDots"),
  );
  assert.equal(appEntry.includes("<TitleBar connectedProvider="), false);
  assert.equal(titleBarSource.includes("connectedProvider"), false);
  assert.equal(appEntry.includes('meta={<ReadOnlyBadge>本地只读</ReadOnlyBadge>}'), false);
  assert.ok(appEntry.includes("进入工作台"));
  assert.ok(appEntry.includes("工作台"));
  assert.ok(appEntry.includes("任务"));
  assert.ok(appEntry.includes("文件"));
  assert.ok(appEntry.includes("交付"));
  assert.ok(appEntry.includes("审计"));
  assert.ok(appEntry.includes("高级"));
  assert.ok(appEntry.includes("复制任务"));
  assert.ok(appEntry.includes("优先级"));
  assert.equal(appEntry.includes("displayRiskLabelZh"), false);
  assert.equal(appEntry.includes("请求人工审计"), false);
  assert.ok(appEntry.includes("等待 Agent 审计"));
  assert.ok(appEntry.includes("displayStatusColumns"));
  assert.ok(stateStatusHook.includes("rebuild_task_projections"));
  assert.ok(stateStatusHook.includes("load_projection_issue_status_index"));
  assert.ok(appEntry.includes('aria-label="任务工作流"'));
  assert.ok(appEntry.includes('aria-label="任务状态流转"'));
  assert.ok(appEntry.includes('aria-label="当前阶段摘要"'));
  assert.ok(appEntry.includes('aria-label="执行与交付"'));
  assert.ok(appEntry.includes('aria-label="审计摘要与入口"'));
  assert.ok(appEntry.includes('aria-label="项目调度视图"'));
  assert.ok(appEntry.includes('aria-label="项目阶段摘要"'));
  assert.ok(appEntry.includes('aria-label="项目状态流"'));
  assert.ok(appEntry.includes('aria-label="项目交付摘要"'));
  assert.ok(appEntry.includes('aria-label="项目审计摘要"'));
  assert.ok(appEntry.includes('aria-label="Spec Loop 阶段"'));
  assert.ok(appEntry.includes('aria-label="预览与物化"'));
  assert.ok(appEntry.includes("状态时间线 / 事件流"));
  assert.ok(appEntry.includes("当前还没有进入最终交付阶段"));
  assert.ok(appEntry.includes("运行 Project Loop"));
  assert.ok(appEntry.includes("执行与交付"));
  assert.ok(appEntry.includes("验收门"));
  assert.ok(appEntry.includes("完成写回"));
  assert.ok(appEntry.includes("Acceptance Gate"));
  assert.ok(appEntry.includes("Completion Commit"));
  assert.ok(appEntry.includes('aria-label="事件时间线与证据图"'));
  assert.ok(appEntry.includes('aria-label="事件时间线"'));
  assert.ok(appEntry.includes('aria-label="证据图"'));
  assert.ok(appEntry.includes("审计是独立旁支，不参与 Done 默认链路"));
  assert.ok(appEntry.includes('aria-label="验收与交付表面"'));
  assert.ok(appEntry.includes("Release readiness"));
  assert.ok(appEntry.includes("Projection refresh 不是 authority"));
  assert.ok(appEntry.includes('aria-label="审计只读表面"'));
  assert.ok(appEntry.includes("Audit Surface 不修改 Work Loop facts"));
  assert.ok(appEntry.includes("audit queued 不等于 audit passed"));
  assert.ok(appEntry.includes("Done 后 no audit 是合法状态"));
  assert.ok(appEntry.includes('aria-label="Command Surface Runtime API Bridge"'));
  assert.ok(appEntry.includes('aria-label="任务 Command Surface"'));
  assert.ok(appEntry.includes("RuntimeCommand(approveSpec) -> ActionProposal -> Arbitration"));
  assert.ok(appEntry.includes("RuntimeCommand(startWork) -> ActionProposal -> Arbitration"));
  assert.ok(appEntry.includes("RuntimeCommand(requestAudit) -> AuditSurfaceView"));
  assert.ok(appEntry.includes("RuntimeCommand(createFollowUp) -> createIssue proposal"));
  assert.ok(appEntry.includes("needs-human-decision"));
  assert.ok(runtimeApiMapping.includes('"acceptDelivery" => Some("markIssueDone")'));
  assert.ok(runtimeApiMapping.includes('"requestFix" | "reopenIssue" => Some("recordDecision")'));
  assert.ok(runtimeApiMapping.includes('"createIssue" | "createFollowUp" => Some("createIssue")'));
  assert.ok(runtimeApiCommands.includes("command_surface_aliases_map_to_supported_action_contracts"));
  assert.ok(appEntry.includes("交付槽位"));
  assert.ok(appEntry.includes("公开交付"));
  assert.ok(appEntry.includes("需求工作台"));
  assert.ok(appEntry.includes("Runtime Action Proposal"));
  assert.ok(appEntry.includes("证据链"));
  assert.ok(appEntry.includes("追溯关系"));
  assert.ok(appEntry.includes("AdvancedStateViewer"));
  assert.ok(appShellCss.includes(".v16-status-bar"));
  assert.ok(appShellCss.includes(".v16-tasks-page"));
  assert.ok(appShellCss.includes(".v16-task-list-layout"));
  assert.ok(appShellCss.includes(".v16-task-queue-row"));
  assert.ok(appShellCss.includes(".v16-spec-layout"));
  assert.ok(appShellCss.includes(".v16-spec-stage-row"));
  assert.ok(appShellCss.includes(".v16-task-evidence-graph"));
  assert.ok(appShellCss.includes(".v16-task-evidence-chain"));
  assert.ok(appShellCss.includes(".v16-task-acceptance-delivery"));
  assert.ok(appShellCss.includes(".v16-task-audit-surface"));
  assert.ok(appShellCss.includes(".v16-command-surface-list"));
  assert.ok(appShellCss.includes(".v16-task-command-surface"));
  assert.ok(appShellCss.includes(".v16-files-page"));
  assert.ok(appShellCss.includes("@media (prefers-color-scheme: dark)"));
  assert.ok(projectLocalFilesPage.indexOf("<ProjectFileBrowser") < projectLocalFilesPage.indexOf("<article className=\"project-file-reader\""));
  assert.ok(designSystemPreview.includes('data-agentflow-design-system="v1"'));
  for (const [marker, relativePath] of designSystemFiles) {
    const componentSource = readFileSync(path.join(desktopRoot, relativePath), "utf8");
    assert.ok(
      componentSource.includes(`data-agentflow-component="${marker}"`),
      `Missing design system marker: ${marker}`,
    );
  }
  assert.equal(existsSync(path.join(smokeRoot, ".agentflow/audit")), false);

  console.log("Browser Preview smoke passed: task page workflow hub, project page summary, current/past/future timeline boundaries, delivery summary, human audit, design system, and V16 shell are read-only.");
} finally {
  await server.close();
}
