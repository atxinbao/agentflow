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
  assert.ok(appEntry.includes("交付摘要"));
  assert.ok(appEntry.includes("证据链"));
  assert.ok(appEntry.includes("追溯关系"));
  assert.ok(appEntry.includes("AdvancedStateViewer"));
  assert.ok(appShellCss.includes(".v16-status-bar"));
  assert.ok(appShellCss.includes(".v16-tasks-page"));
  assert.ok(appShellCss.includes(".v16-task-list-layout"));
  assert.ok(appShellCss.includes(".v16-task-queue-row"));
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

  console.log("Browser Preview smoke passed: workflow state, task delivery projection, human audit, design system, and V16 shell are read-only.");
} finally {
  await server.close();
}
