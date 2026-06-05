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
  const outputIndex = preview.createBrowserPreviewOutputIndex();
  const auditIndex = preview.createBrowserPreviewAuditIndex();
  const auditReport = preview.createBrowserPreviewHumanAuditReport();
  const agentEnvironment = preview.createBrowserPreviewAgentEnvironmentStatus(smokeRoot);
  const stateStatus = preview.createBrowserPreviewStateStatus(smokeRoot);
  const outputPanel = readFileSync(
    path.join(desktopRoot, "src/features/output/OutputAuditPanel.tsx"),
    "utf8",
  );
  const previewBranchIndex = outputPanel.indexOf("if (isBrowserPreviewRuntime()) {");
  const previewOnlyGuardIndex = outputPanel.indexOf("if (previewOnly) {");
  const requestAuditInvokeIndex = outputPanel.indexOf('invoke<HumanAuditReport>("request_human_audit"');

  assert.equal(outputStatus.ready, true);
  assert.equal(outputStatus.summary.evidence, 1);
  assert.equal(outputStatus.summary.releaseDeliveries, 1);
  assert.equal(outputStatus.summary.audits, 1);
  assert.equal(outputStatus.summary.incompleteEvidence, 0);
  assert.equal(outputStatus.summary.incompleteDeliveries, 0);
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
  assert.equal(existsSync(path.join(smokeRoot, ".agentflow/output/audit")), false);

  console.log("Browser Preview smoke passed: workflow state and human audit preview are read-only.");
} finally {
  await server.close();
}
