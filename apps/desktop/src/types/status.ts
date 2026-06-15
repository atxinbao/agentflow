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

export type IssueDisplayStatus = "backlog" | "blocked" | "todo" | "in_progress" | "in_review" | "done" | "cancel";

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

export type InputIssueStatus = "backlog" | "todo" | "in_progress" | "in_review" | "done" | "blocked" | "cancel";
export type IssueCategory = "spec" | "audit";
export type AgentRole = "spec-agent" | "build-agent" | "audit-agent";
export type ExpectedOutputs = Record<string, string>;

export type ExecutionPipelineStage = {
  stageId: string;
  label: string;
  goal: string;
  required: boolean;
  evidence: string[];
};

export type ExecutionPipeline = {
  version: string;
  agentRole: AgentRole;
  gitProviders?: string[];
  stages: ExecutionPipelineStage[];
  mergeModes: string[];
};

export type McpLaunchMode =
  | "cli-exec-stdin"
  | "cli-exec-prompt-file"
  | "app-server-thread"
  | "mcp-remote-session";

export type McpSessionStatus =
  | "queued"
  | "claimed"
  | "starting"
  | "running"
  | "in-review"
  | "done"
  | "failed"
  | "cancelled";

export type McpSessionSnapshot = {
  version: string;
  provider: string;
  issueId: string;
  projectId?: string | null;
  runId: string;
  sessionId: string;
  status: McpSessionStatus;
  launchMode: McpLaunchMode;
  launchRequestPath: string;
  planPath: string;
  logPath?: string | null;
  branchName?: string | null;
  pid?: number | null;
  remoteSessionId?: string | null;
  prUrl?: string | null;
  mergeState?: string | null;
  note?: string | null;
  lastError?: string | null;
  createdAt: number;
  updatedAt: number;
};

export type McpLogChunk = {
  version: string;
  provider: string;
  sessionId: string;
  cursor?: string | null;
  lines: string[];
};

export type InputProject = {
  version: string;
  projectId: string;
  sourceSpecId?: string | null;
  title: string;
  summary: string;
  objective?: string | null;
  scope: string[];
  nonGoals: string[];
  successCriteria: string[];
  issueIds: string[];
  status: string;
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

export type InputIssueRelationType = "blocked-by" | "blocks" | "related" | "duplicate-of";

export type InputIssueRelationEdge = {
  fromIssueId: string;
  toIssueId: string;
  type: InputIssueRelationType;
};

export type InputIssueRelations = {
  version: string;
  relations?: InputIssueRelationEdge[];
  edges?: InputIssueRelationEdge[];
  nodes?: string[];
};

export type InputIssue = {
  version: string;
  issueId: string;
  issueModel: "direct" | "project";
  issueCategory?: IssueCategory;
  requiredAgentRole?: AgentRole;
  sourceSpecId: string;
  sourceSpecPath?: string;
  issuePath?: string;
  handoffId?: string;
  contextPackPath?: string;
  projectId?: string | null;
  title: string;
  summary: string;
  kind: string;
  priority: string;
  status: InputIssueStatus;
  displayStatus: IssueDisplayStatus;
  executionRisk: string;
  allowedPaths?: string[];
  forbiddenPaths?: string[];
  forbiddenActions?: string[];
  scope: string[];
  nonGoals: string[];
  acceptanceCriteria: string[];
  validationHints: string[];
  validationCommands?: string[];
  expectedOutputs?: ExpectedOutputs;
  executionPipeline?: ExecutionPipeline | null;
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
  audit?: {
    auditId?: string;
    trigger?: string;
    sourceReleaseId?: string;
    sourceRunId?: string | null;
    sourceDeliveryPath?: string;
    auditOutputDir?: string;
    expectedOutputs?: ExpectedOutputs | string[];
  } | null;
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
  projects: InputProject[];
  issues: InputIssue[];
  relations: InputIssueRelations;
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
  publicDeliveries: number;
  audits: number;
  logs: number;
  backups: number;
  incompleteEvidence: number;
  incompletePublicDeliveries: number;
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
  audits: OutputIndexEntry[];
};

export type AuditStatus = "requested" | "running" | "passed" | "passed-with-warnings" | "failed" | "cancelled";
export type AuditTrigger = "human-via-agent" | "release-auto";

export type AuditIndexEntry = {
  auditId: string;
  status: AuditStatus;
  trigger?: AuditTrigger;
  requestedBy: string;
  requestedAt: number;
  sourceDeliveryId?: string | null;
  sourceRunId?: string | null;
  sourceIssueId?: string | null;
  sourceSpecId?: string | null;
  reportPath: string;
  auditPath: string;
};

export type AuditIndex = {
  version: string;
  updatedAt: number;
  audits: AuditIndexEntry[];
};

export type HumanAuditReport = {
  request: {
    trigger?: AuditTrigger;
    source?: {
      kind?: string;
      deliveryId?: string | null;
      runId?: string | null;
      issueId?: string | null;
      specId?: string | null;
    } | null;
    [key: string]: unknown;
  };
  audit: {
    auditId: string;
    status: AuditStatus;
    trigger?: AuditTrigger;
    requestedBy: string;
    requestedAt: number;
    sourceDeliveryId?: string | null;
    sourceRunId?: string | null;
    sourceIssueId?: string | null;
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
  priority?: string;
  executionRisk?: string;
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

export type ProjectionPhase = "past" | "current" | "future" | "exception";

export type TaskTimelineItem = {
  state: IssueDisplayStatus;
  phase: ProjectionPhase;
  enteredAt?: number | null;
  events: string[];
  summary: string;
  liveRefs: string[];
};

export type ProjectionPublicDelivery = {
  evidencePath?: string | null;
  prUrl?: string | null;
  mergeCommit?: string | null;
  changelogPath?: string | null;
  releaseNotesUrl?: string | null;
};

export type TaskProjection = {
  version: string;
  issueId: string;
  projectId?: string | null;
  workflowRef: string;
  currentState: IssueDisplayStatus;
  displayStatus: IssueDisplayStatus;
  currentTransition?: string | null;
  latestRunId?: string | null;
  branchName?: string | null;
  timeline: TaskTimelineItem[];
  publicDelivery: ProjectionPublicDelivery;
  updatedAt: number;
};

export type ProjectProjection = {
  version: string;
  projectId: string;
  title: string;
  status: string;
  issueIds: string[];
  currentIssueId?: string | null;
  issueCount: number;
  completedIssueCount: number;
  updatedAt: number;
};

export type ProjectionSummary = {
  taskCount: number;
  projectCount: number;
  indexPath: string;
};
