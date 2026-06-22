import assert from "node:assert/strict";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");
const readinessRoot = mkdtempSync(path.join(tmpdir(), "agentflow-console-readiness-"));
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
  const inputSnapshot = preview.createBrowserPreviewInputSnapshot(readinessRoot);
  const issueStatusIndex = preview.createBrowserPreviewIssueStatusIndex(readinessRoot);
  const taskProjections = inputSnapshot.issues
    .map((issue) => preview.createBrowserPreviewTaskProjection(issue.issueId, readinessRoot, "default"))
    .filter(Boolean);
  const selectedTaskProjection = preview.createBrowserPreviewTaskProjection("iss-progress", readinessRoot, "default");
  const projectProjection = preview.createBrowserPreviewProjectProjection(
    "project-browser-preview",
    readinessRoot,
    "default",
  );
  const specWorkbenchProjection = preview.createBrowserPreviewSpecWorkbenchProjection();
  const taskTree = viewModels.buildTaskProjectTreeViewModel({
    activeIssueId: "iss-progress",
    issues: inputSnapshot.issues,
    issueStatusIndex,
    projects: inputSnapshot.projects,
    relations: inputSnapshot.relations,
  });
  const desktopViewModels = viewModels.buildDesktopProjectionViewModels({
    projectProjection,
    selectedTaskProjection,
    specWorkbenchProjection,
    taskProjections,
    taskTree,
  });
  const missingProject = viewModels.buildProjectHomeViewModel(null);
  const staleProject = viewModels.buildProjectHomeViewModel({
    ...projectProjection,
    projectBrain: {
      ...projectProjection.projectBrain,
      missingDocuments: ["docs/foundation/project-brain.md"],
    },
  });
  const conflictSpec = viewModels.buildSpecWorkbenchViewModel({
    ...specWorkbenchProjection,
    warnings: ["conflict: preview artifact 与 issue authority 不一致。"],
  });

  assert.equal(desktopViewModels.version, "desktop-projection-view-models.v1");
  assert.equal(desktopViewModels.projectHome.projectId, "project-browser-preview");
  assert.equal(desktopViewModels.projectHome.readiness.status, "ready");
  assert.equal(desktopViewModels.specWorkbench.stageCount, 8);
  assert.equal(desktopViewModels.specWorkbench.runtimeActionProposalCount, 1);
  assert.equal(desktopViewModels.taskWorkbench.selectedIssueId, "iss-progress");
  assert.equal(desktopViewModels.taskWorkbench.timelineStates.includes("in_progress"), true);
  assert.equal(desktopViewModels.taskWorkbench.evidenceGraphState, "done");
  assert.equal(desktopViewModels.taskWorkbench.commandSurfaceState, "ready");
  assert.equal(desktopViewModels.acceptanceDeliveryAudit.acceptanceState, "ready");
  assert.equal(desktopViewModels.acceptanceDeliveryAudit.deliveryState, "ready");
  assert.equal(desktopViewModels.acceptanceDeliveryAudit.auditState, "ready");
  assert.equal(desktopViewModels.surfaces.every((surface) => surface.readonly), true);
  assert.deepEqual(
    desktopViewModels.surfaces.map((surface) => surface.id),
    [
      "project-home",
      "spec-workbench",
      "task-workbench",
      "event-timeline",
      "evidence-graph",
      "acceptance-delivery-audit",
      "command-surface",
    ],
  );
  assert.equal(missingProject.readiness.status, "missing");
  assert.equal(staleProject.readiness.status, "stale");
  assert.equal(conflictSpec.readiness.status, "conflict");

  const summary = {
    acceptanceDeliveryAudit: desktopViewModels.acceptanceDeliveryAudit.readiness.status,
    commandSurface: desktopViewModels.taskWorkbench.commandSurfaceState,
    projectHome: desktopViewModels.projectHome.readiness.status,
    readonlySurfaces: desktopViewModels.surfaces.length,
    specWorkbench: desktopViewModels.specWorkbench.readiness.status,
    taskWorkbench: desktopViewModels.taskWorkbench.readiness.status,
  };
  console.log(`Project OS Console readiness passed: ${JSON.stringify(summary)}`);
} finally {
  await server.close();
}
