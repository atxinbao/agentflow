export type WorkbenchSnapshot = {
  version: string;
  initialized: boolean;
  projectRoot: string;
  projectSummaryMarkdown?: string | null;
  goalLoopSummaryMarkdown?: string | null;
  goalLoop?: GoalLoopState | null;
  issues: IssueContract[];
  runs: AgentRun[];
  savedViews: SavedView[];
  evidence: WorkbenchTextArtifact[];
  reviews: WorkbenchTextArtifact[];
  projectUpdates: WorkbenchTextArtifact[];
  counts: WorkbenchCounts;
  boundary: WorkbenchBoundary;
};

export type GoalLoopState = {
  version: string;
  goalReady: boolean;
  activeIssueId?: string | null;
  incompleteIssues: GoalLoopIssueRef[];
  nextAction: string;
  recommendedIssueIntent: string;
  recommendedCommand: string;
  rationale: string[];
  counts: GoalLoopCounts;
  sources: Record<string, string>;
};

export type GoalLoopIssueRef = {
  id: string;
  title: string;
  status: string;
  nextAction: string;
};

export type GoalLoopCounts = {
  issues: number;
  completedIssues: number;
  runs: number;
  evidenceReports: number;
  reviews: number;
  projectUpdates: number;
};

export type WorkbenchCounts = {
  issues: number;
  completedIssues: number;
  runs: number;
  passedRuns: number;
  evidenceReports: number;
  reviews: number;
  projectUpdates: number;
  savedViews: number;
};

export type IssueContract = {
  id: string;
  title: string;
  status: string;
  intent: string;
  scope: string[];
  nonGoals: string[];
  context: {
    repo: string;
    files: string[];
  };
  executionPlan: string[];
  validation: {
    commands: string[];
  };
  evidenceRequirements: string[];
  humanGate: {
    beforeFileEdits: boolean;
    beforeExternalNetwork: boolean;
  };
  aep?: {
    phase: string;
    stopCondition: string;
    fastestFeedbackLoop: string[];
    verticalSlice: string;
    tracerBulletPlan: string[];
    diagnosePlan: string[];
    panelContextStatus: string;
    docsClaimTrace: string[];
    boundaryConfirmation: string[];
    prHandoffRequirements: string[];
  };
};

export type AgentRun = {
  id: string;
  issueId: string;
  status: string;
  mode: string;
  validationCommands: CommandRecord[];
  outputs: {
    transcript: string;
    commands: string;
    diffSummary: string;
    evidence?: string | null;
    review?: string | null;
    update?: string | null;
  };
};

export type CommandRecord = {
  command: string;
  exitCode: number;
  status: string;
  stdout: string;
  stderr: string;
};

export type SavedView = {
  version: string;
  id: string;
  name: string;
  filter: {
    issueStatus?: string | null;
    runStatus?: string | null;
    validationStatus?: string | null;
    issueId?: string | null;
  };
};

export type WorkbenchTextArtifact = {
  path: string;
  title: string;
  content: string;
};

export type WorkbenchBoundary = {
  readOnly: boolean;
  disallowedActions: string[];
};

export type LocalMetricsSnapshot = {
  version: string;
  initialized: boolean;
  projectRoot: string;
  issues: LocalIssueMetrics;
  runs: LocalRunMetrics;
  artifacts: LocalArtifactMetrics;
  goalReady: boolean;
  activeIssueId?: string | null;
  nextAction: string;
  recommendedCommand: string;
  latestRun?: LocalMetricRunRef | null;
  latestEvidence?: LocalMetricArtifactRef | null;
  latestReview?: LocalMetricArtifactRef | null;
  sources: string[];
  boundary: WorkbenchBoundary;
};

export type LocalIssueMetrics = {
  total: number;
  completed: number;
  planned: number;
  active: number;
};

export type LocalRunMetrics = {
  total: number;
  passed: number;
  failed: number;
  missingValidation: number;
};

export type LocalArtifactMetrics = {
  evidenceReports: number;
  reviews: number;
  projectUpdates: number;
  savedViews: number;
};

export type LocalMetricRunRef = {
  id: string;
  issueId: string;
  status: string;
  validationStatus: string;
};

export type LocalMetricArtifactRef = {
  path: string;
  title: string;
};
