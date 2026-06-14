import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewInputSnapshot,
  currentBrowserPreviewTaskHierarchyScenario,
} from "../../../browserPreviewData";
import type {
  AgentRole,
  ExpectedOutputs,
  InputIssue,
  InputIssueStatus,
  InputProject,
  InputSnapshot,
  InputStatusSnapshot,
  IssueCategory,
} from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type InputSnapshotState = {
  snapshot: InputSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialInputSnapshotState: InputSnapshotState = {
  snapshot: null,
  error: null,
  source: "idle",
};

type SpecExpectedOutputs = {
  taskRunDir: string;
  evidencePath: string;
  publicDeliveryRecord?: {
    prOrMrBody?: boolean;
    changelogOrReleaseNotes?: string;
  };
};

type SpecSystemRecord = {
  createdBy: string;
  createdAt: number;
  updatedAt: number;
  path: string;
  publicRequirementPath: string;
};

type SpecIssue = {
  version: string;
  issueId: string;
  issueCategory: IssueCategory;
  requiredAgentRole: AgentRole;
  status: InputIssueStatus;
  workflowRef: string;
  sourceRequirementId: string;
  sourceRequirementPath: string;
  sourceSpecId: string;
  projectId?: string | null;
  title: string;
  summary: string;
  priority: string;
  blockedBy: string[];
  allowedPaths: string[];
  forbiddenPaths: string[];
  validationCommands: string[];
  expectedOutputs: SpecExpectedOutputs;
  system: SpecSystemRecord;
};

type SpecProject = {
  version: string;
  projectId: string;
  sourceRequirementId: string;
  sourceRequirementPath: string;
  title: string;
  summary: string;
  objective: string;
  issueIds: string[];
  status: string;
  system: SpecSystemRecord;
};

type DesktopSpecTaskSnapshot = {
  version: string;
  projectRoot: string;
  projects: SpecProject[];
  issues: SpecIssue[];
  updatedAt: number;
};

async function loadInputSnapshotWithRepair(projectRoot: string) {
  const snapshot = await invoke<DesktopSpecTaskSnapshot>("load_spec_task_snapshot", { projectRoot });
  return specTaskSnapshotToInputSnapshot(snapshot);
}

function specTaskSnapshotToInputSnapshot(snapshot: DesktopSpecTaskSnapshot): InputSnapshot {
  const status = specSnapshotStatus(snapshot);
  return {
    version: "input-snapshot-from-spec.v1",
    projectRoot: snapshot.projectRoot,
    ready: true,
    status,
    manifest: { source: "spec", version: snapshot.version },
    index: {
      source: "spec",
      updatedAt: snapshot.updatedAt,
      projects: snapshot.projects.map((project) => project.projectId),
      issues: snapshot.issues.map((issue) => issue.issueId),
    },
    intake: [],
    specs: [],
    projects: snapshot.projects.map(specProjectToInputProject),
    issues: snapshot.issues.map(specIssueToInputIssue),
    relations: {
      version: "spec-derived-relations.v1",
      edges: snapshot.issues.flatMap((issue) =>
        issue.blockedBy.map((blockedBy) => ({
          fromIssueId: issue.issueId,
          toIssueId: blockedBy,
          type: "blocked-by" as const,
        })),
      ),
      nodes: snapshot.issues.map((issue) => issue.issueId),
    },
  };
}

function specSnapshotStatus(snapshot: DesktopSpecTaskSnapshot): InputStatusSnapshot {
  return {
    version: "input-status-from-spec.v1",
    projectRoot: snapshot.projectRoot,
    status: "ready",
    ready: true,
    manifestExists: true,
    indexExists: true,
    summary: {
      intake: 0,
      draftSpecs: 0,
      approvedSpecs: snapshot.projects.length,
      projects: snapshot.projects.length,
      issues: snapshot.issues.length,
      blockedIssues: snapshot.issues.filter((issue) => issue.status === "blocked").length,
      highRiskIssues: 0,
    },
    missingPaths: [],
    warnings: [],
    errors: [],
  };
}

function specProjectToInputProject(project: SpecProject): InputProject {
  return {
    version: project.version,
    projectId: project.projectId,
    sourceSpecId: project.sourceRequirementId,
    title: project.title,
    summary: project.summary,
    objective: project.objective,
    scope: [],
    nonGoals: [],
    successCriteria: [],
    issueIds: project.issueIds,
    status: project.status,
    system: {
      createdBy: project.system.createdBy,
      createdAt: project.system.createdAt,
      updatedAt: project.system.updatedAt,
      path: project.system.path,
    },
  };
}

function specIssueToInputIssue(issue: SpecIssue): InputIssue {
  return {
    version: issue.version,
    issueId: issue.issueId,
    issueModel: issue.projectId ? "project" : "direct",
    issueCategory: issue.issueCategory,
    requiredAgentRole: issue.requiredAgentRole,
    sourceSpecId: issue.sourceSpecId,
    sourceSpecPath: issue.sourceRequirementPath,
    issuePath: issue.system.path,
    handoffId: `handoff-${issue.issueId}`,
    contextPackPath: `.agentflow/panel/context-packs/${issue.issueId}/context-pack.json`,
    projectId: issue.projectId ?? null,
    title: issue.title,
    summary: issue.summary,
    kind: issue.issueCategory === "audit" ? "audit" : "spec",
    priority: issue.priority,
    status: issue.status,
    displayStatus: issue.status,
    executionRisk: "low",
    allowedPaths: issue.allowedPaths,
    forbiddenPaths: issue.forbiddenPaths,
    forbiddenActions: ["do-not-write-agentflow-facts-by-hand"],
    scope: issue.allowedPaths,
    nonGoals: issue.forbiddenPaths,
    acceptanceCriteria: issue.validationCommands,
    validationHints: issue.validationCommands,
    validationCommands: issue.validationCommands,
    expectedOutputs: specExpectedOutputsToRecord(issue.expectedOutputs),
    executionPipeline: null,
    relations: {
      blockedBy: issue.blockedBy,
    },
    system: {
      createdBy: issue.system.createdBy,
      createdAt: issue.system.createdAt,
      updatedAt: issue.system.updatedAt,
      path: issue.system.path,
    },
  };
}

function specExpectedOutputsToRecord(outputs: SpecExpectedOutputs): ExpectedOutputs {
  return {
    taskRunDir: outputs.taskRunDir,
    evidencePath: outputs.evidencePath,
    publicDeliveryRecord: outputs.publicDeliveryRecord?.changelogOrReleaseNotes ?? "required-when-release-visible",
  };
}

export function useInputSnapshot(projectRoot: string | null, refreshToken = 0) {
  const [inputSnapshotState, setInputSnapshotState] =
    useState<InputSnapshotState>(initialInputSnapshotState);

  useEffect(() => {
    if (!projectRoot) {
      setInputSnapshotState(initialInputSnapshotState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      const scenario = currentBrowserPreviewTaskHierarchyScenario();
      setInputSnapshotState({
        snapshot: createBrowserPreviewInputSnapshot(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT, scenario),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setInputSnapshotState((current) =>
      current.snapshot ? { ...current, error: null } : { ...current, error: null, source: "loading" },
    );
    void loadInputSnapshotWithRepair(projectRoot)
      .then((snapshot) => {
        if (!cancelled) {
          setInputSnapshotState({ snapshot, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : String(error);
          setInputSnapshotState((current) =>
            current.snapshot
              ? { ...current, error: message }
              : {
                  snapshot: null,
                  error: message,
                  source: "unavailable",
                },
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, refreshToken]);

  return inputSnapshotState;
}
