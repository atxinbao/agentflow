export type {
  AgentStatusChannelItem,
  AgentStatusMetric,
  AgentStatusTone,
} from "../features/status-channel/statusTypes";

export type AgentEnvironmentState =
  | "missing"
  | "checking"
  | "repairing"
  | "ready"
  | "repaired"
  | "degraded"
  | "failed"
  | "blocked";

export type AgentEnvironmentStatus = {
  version: string;
  projectRoot: string;
  status: AgentEnvironmentState;
  ready: boolean;
  checkedAt: number;
  repairedAt?: number | null;
  agentMd: {
    exists: boolean;
    managed: boolean;
    version?: string | null;
    hash?: string | null;
    backedUp: boolean;
    trackedByGit: boolean;
  };
  manual: {
    exists: boolean;
    path: string;
    hash?: string | null;
  };
  skillsLock: {
    exists: boolean;
    valid: boolean;
    path: string;
    skillCount: number;
  };
  skills: Array<{
    name: string;
    path: string;
    exists: boolean;
    hashMatches: boolean;
    version: string;
  }>;
  repairs: string[];
  warnings: string[];
  errors: string[];
  workspaceManifest: {
    exists: boolean;
    path: string;
    valid: boolean;
    layoutVersion?: string | null;
  };
  ownership: WorkspaceOwnershipStatus;
  layout: {
    version: string;
    ready: boolean;
    createdPaths: string[];
    reusedPaths: string[];
    missingPaths: string[];
  };
  locale: {
    version: string;
    agentLocale: string;
    rawOsLocale?: string | null;
    manualLanguage: string;
    source: string;
    checkedAt: number;
    fallback: boolean;
    warnings: string[];
  };
  style: {
    version: string;
    styleId: string;
    manualLanguage: string;
    appliesToAgentLocale: boolean;
    appliesToCodeComments: boolean;
    checkedAt: number;
    warnings: string[];
  };
  legacyAgentEntry: {
    exists: boolean;
    path: string;
    managed: boolean;
  };
  shadowGuard: {
    checked: string[];
    detected: string[];
  };
};

export type WorkspaceOwnershipState =
  | "none"
  | "managed-current"
  | "managed-legacy"
  | "foreign"
  | "corrupted"
  | "blocked";

export type WorkspaceOwnershipAction =
  | "create"
  | "validate-repair"
  | "migrate-repair"
  | "ask-user-to-take-over"
  | "stop";

export type WorkspaceOwnershipStatus = {
  version: string;
  projectRoot: string;
  status: WorkspaceOwnershipState;
  readyForPrepare: boolean;
  agentBlocked: boolean;
  agentflowPath: string;
  marker: {
    manifestExists: boolean;
    manifestManagedByAgentflow: boolean;
    manifestVersion?: string | null;
    layoutVersion?: string | null;
    agentManualExists: boolean;
    skillsLockExists: boolean;
    managedEntryExists: boolean;
  };
  detectedFiles: string[];
  warnings: string[];
  errors: string[];
  recommendedAction: WorkspaceOwnershipAction;
};

export type InputWorkspaceStatus = "missing" | "ready" | "degraded" | "failed" | "blocked";

export type IssueDisplayStatus = "backlog" | "ready" | "in-progress" | "review" | "done" | "cancel";

export type InputSummary = {
  intake: number;
  draftSpecs: number;
  approvedSpecs: number;
  projects: number;
  issues: number;
  blockedIssues: number;
  highRiskIssues: number;
};

export type InputStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: InputWorkspaceStatus;
  ready: boolean;
  manifestExists: boolean;
  indexExists: boolean;
  summary: InputSummary;
  missingPaths: string[];
  warnings: string[];
  errors: string[];
};

export type InputIssueStatus = "planned" | "blocked" | "ready-for-execute" | "done" | "canceled";

export type InputIssue = {
  version: string;
  issueId: string;
  issueModel: "direct" | "project";
  sourceSpecId: string;
  projectId?: string | null;
  title: string;
  summary: string;
  kind: string;
  priority: string;
  status: InputIssueStatus;
  displayStatus: IssueDisplayStatus;
  riskLevel: string;
  scope: string[];
  nonGoals: string[];
  acceptanceCriteria: string[];
  validationHints: string[];
  relations?: {
    blockedBy?: string[];
    blocks?: string[];
    related?: string[];
    duplicateOf?: string | null;
  };
  panel?: {
    snapshotId?: string | null;
    contextPackId?: string | null;
  };
  system?: {
    createdBy?: string;
    createdAt?: number;
    updatedAt?: number;
    path?: string;
    revision?: number;
  };
};

export type InputSnapshot = {
  version: string;
  projectRoot: string;
  ready: boolean;
  status: InputStatusSnapshot;
  manifest: unknown;
  index: unknown;
  intake: unknown[];
  specs: unknown[];
  projects: unknown[];
  issues: InputIssue[];
  relations: unknown;
};

export type ExecuteWorkspaceStatus = "missing" | "ready" | "degraded" | "failed" | "blocked";

export type ExecuteSummary = {
  runs: number;
  activeRuns: number;
  blockedRuns: number;
  completedRuns: number;
  activeLeases: number;
};

export type ExecuteStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: ExecuteWorkspaceStatus;
  ready: boolean;
  manifestExists: boolean;
  indexExists: boolean;
  summary: ExecuteSummary;
  missingPaths: string[];
  warnings: string[];
  errors: string[];
};

export type OutputWorkspaceStatus = "missing" | "ready" | "degraded" | "failed" | "blocked";

export type OutputSummary = {
  evidence: number;
  releaseDeliveries: number;
  audits: number;
  logs: number;
  backups: number;
  incompleteEvidence: number;
  incompleteDeliveries: number;
};

export type OutputStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: OutputWorkspaceStatus;
  ready: boolean;
  manifestExists: boolean;
  indexExists: boolean;
  summary: OutputSummary;
  missingPaths: string[];
  warnings: string[];
  errors: string[];
};

export type OutputIndexEntry = {
  runId: string;
  issueId: string;
  sourceSpecId: string;
  path: string;
  status: string;
  updatedAt: number;
};

export type OutputIndex = {
  version: string;
  updatedAt: number;
  evidence: OutputIndexEntry[];
  releaseDeliveries: OutputIndexEntry[];
  audits: OutputIndexEntry[];
};

export type AuditStatus = "passed" | "passed-with-warnings" | "failed" | "cancelled";

export type AuditIndexEntry = {
  auditId: string;
  status: AuditStatus;
  requestedBy: string;
  requestedAt: number;
  reportPath: string;
  auditPath: string;
};

export type AuditIndex = {
  version: string;
  updatedAt: number;
  audits: AuditIndexEntry[];
};

export type AuditScopeRef = {
  kind: string;
  id: string;
  path: string;
};

export type AuditScope = {
  description: string;
  refs: AuditScopeRef[];
};

export type HumanAuditRequestDraft = {
  reason: string;
  scope: AuditScope;
};

export type HumanAuditReport = {
  request: unknown;
  audit: {
    auditId: string;
    status: AuditStatus;
    requestedBy: string;
    requestedAt: number;
    summary?: unknown;
    checks?: unknown;
    paths?: Record<string, string>;
  };
  reportMarkdown: string;
  findings: unknown;
  checklistMarkdown: string;
  evidenceMap: unknown;
  traceability: unknown;
};

export type StateWorkspaceStatus = "missing" | "ready" | "degraded" | "failed" | "blocked";

export type WorkflowStage =
  | "workspace-missing"
  | "workspace-blocked"
  | "workspace-ready"
  | "panel-ready"
  | "input-ready"
  | "issue-ready"
  | "execute-ready"
  | "execute-running"
  | "execute-blocked"
  | "execute-completed"
  | "evidence-ready"
  | "delivery-ready"
  | "audit-requested"
  | "audit-running"
  | "audit-completed"
  | "failed";

export type WorkflowAuditStatus =
  | "not-requested"
  | "requested"
  | "running"
  | "passed"
  | "passed-with-warnings"
  | "failed"
  | "cancelled";

export type WorkflowBlockedAction = {
  action: string;
  reason: string;
  sourcePath?: string | null;
};

export type StateStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: StateWorkspaceStatus;
  currentStage: WorkflowStage;
  auditStatus: WorkflowAuditStatus;
  activeIssueId?: string | null;
  activeRunId?: string | null;
  health: Record<string, string>;
  nextActions: string[];
  blockers: WorkflowBlockedAction[];
  updatedAt: number;
};

export type IssueStatusIndexEntry = {
  issueId: string;
  displayStatus: IssueDisplayStatus;
  riskLevel: string;
  latestRunId?: string | null;
  executeStatus?: string | null;
  evidenceStatus: string;
  deliveryStatus: string;
  auditStatus: WorkflowAuditStatus;
};

export type IssueStatusIndex = {
  version: string;
  updatedAt: number;
  issues: IssueStatusIndexEntry[];
};
