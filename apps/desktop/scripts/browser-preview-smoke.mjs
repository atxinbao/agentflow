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
  const outputStatus = preview.createBrowserPreviewOutputStatus(smokeRoot);
  const inputSnapshot = preview.createBrowserPreviewInputSnapshot(smokeRoot);
  const issueStatusIndex = preview.createBrowserPreviewIssueStatusIndex(smokeRoot);
  const outputIndex = preview.createBrowserPreviewOutputIndex();
  const auditIndex = preview.createBrowserPreviewAuditIndex();
  const auditReport = preview.createBrowserPreviewHumanAuditReport();
  const agentEnvironment = preview.createBrowserPreviewAgentEnvironmentStatus(smokeRoot);
  const stateStatus = preview.createBrowserPreviewStateStatus(smokeRoot);
  const outputPanel = readFileSync(
    path.join(desktopRoot, "src/features/output/OutputAuditPanel.tsx"),
    "utf8",
  );
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
  const previewBranchIndex = outputPanel.indexOf("if (isBrowserPreviewRuntime()) {");
  const previewOnlyGuardIndex = outputPanel.indexOf("if (previewOnly) {");
  const requestAuditInvokeIndex = outputPanel.indexOf('invoke<HumanAuditReport>("request_human_audit"');

  assert.equal(outputStatus.ready, true);
  assert.equal(outputStatus.summary.evidence, 1);
  assert.equal(outputStatus.summary.releaseDeliveries, 1);
  assert.equal(outputStatus.summary.audits, 1);
  assert.equal(outputStatus.summary.incompleteEvidence, 0);
  assert.equal(outputStatus.summary.incompleteDeliveries, 0);
  assert.equal(inputSnapshot.issues.length, 6);
  assert.deepEqual(
    inputSnapshot.issues.map((issue) => issue.displayStatus),
    ["backlog", "ready", "in-progress", "review", "done", "cancel"],
  );
  assert.deepEqual(
    issueStatusIndex.issues.map((issue) => issue.displayStatus),
    ["backlog", "ready", "in-progress", "review", "done", "cancel"],
  );
  assert.equal(outputIndex.releaseDeliveries.length, 1);
  assert.equal(outputIndex.releaseDeliveries[0].runId, "run-browser-preview-001");
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
  assert.ok(previewBranchIndex >= 0);
  assert.ok(outputPanel.includes("setReport(createBrowserPreviewHumanAuditReport())"));
  assert.ok(outputPanel.includes('setSource("preview")'));
  assert.ok(outputPanel.includes('const previewOnly = source === "preview";'));
  assert.ok(outputPanel.includes("previewOnly || !selectedDelivery"));
  assert.ok(previewOnlyGuardIndex >= 0);
  assert.ok(requestAuditInvokeIndex > previewOnlyGuardIndex);
  assert.ok(outputPanel.includes("浏览器预览不写 .agentflow/output/audit"));
  assert.ok(appEntry.includes("<DesignSystemPreview />"));
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
  assert.ok(appEntry.includes("复制任务包"));
  assert.ok(appEntry.includes("请求人工审计"));
  assert.ok(appEntry.includes("displayStatusColumns"));
  assert.ok(stateStatusHook.includes("load_issue_status_index"));
  assert.ok(appEntry.includes("交付摘要"));
  assert.ok(appEntry.includes("证据映射"));
  assert.ok(appEntry.includes("追溯关系"));
  assert.ok(appEntry.includes("AdvancedStateViewer"));
  assert.ok(appShellCss.includes(".v16-status-bar"));
  assert.ok(appShellCss.includes(".v16-task-board"));
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
  assert.equal(existsSync(path.join(smokeRoot, ".agentflow/output/audit")), false);

  console.log("Browser Preview smoke passed: workflow state, human audit, design system, and V16 shell are read-only.");
} finally {
  await server.close();
}
