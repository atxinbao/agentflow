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
    graphifyContextStatus: string;
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

export type LocalProjectModelSnapshot = {
  version: string;
  initialized: boolean;
  projectRoot: string;
  workspace?: LocalWorkspace | null;
  teams: LocalTeam[];
  projects: LocalProject[];
  issueRefs: LocalProjectIssueRef[];
  goalLoopSelection: GoalLoopSelection;
  sources: string[];
  boundary: WorkbenchBoundary;
};

export type LocalWorkspace = {
  version: string;
  id: string;
  name: string;
  defaultTeamId: string;
  activeProjectId: string;
  teamIds: string[];
  projectIds: string[];
  issueCount: number;
  completedIssueCount: number;
};

export type LocalTeam = {
  version: string;
  id: string;
  name: string;
  workflow: string[];
  defaultValidationCommands: string[];
  wipLimit: number;
  issueIds: string[];
};

export type LocalProject = {
  version: string;
  id: string;
  name: string;
  status: string;
  canonicalStatus?: string;
  goal: string;
  teamIds: string[];
  activeMilestoneId: string;
  milestones: LocalMilestone[];
  issueIds: string[];
  issueCount: number;
  completedIssueCount: number;
  nextIssueIntent?: string | null;
  recommendedCommand?: string | null;
};

export type LocalMilestone = {
  id: string;
  name: string;
  description?: string | null;
  sortOrder?: number;
  target?: string | null;
  status: string;
  progress?: MilestoneDerivedProgress;
  issueIds: string[];
  completedIssueIds: string[];
  nextIssueIntent?: string | null;
};

export type MilestoneDerivedProgress = {
  doneIssueCount: number;
  totalIssueCount: number;
  nonCanceledIssueCount: number;
  canceledIssueCount: number;
  percent: number;
};

export type LocalProjectIssueRef = {
  id: string;
  title: string;
  status: string;
  canonicalStatus?: string;
  nextAction: string;
  latestRunId?: string | null;
  latestRunStatus?: string | null;
  validationStatus: string;
  executionState: string;
  evidencePath?: string | null;
  reviewPath?: string | null;
  projectUpdatePath?: string | null;
};

export type GoalLoopSelection = {
  activeProjectId?: string | null;
  source: string;
  nextAction: string;
  nextIssueIntent?: string | null;
  recommendedCommand: string;
  rationale: string[];
};

export type ProjectMilestoneIssueViewModelSnapshot = {
  version: string;
  initialized: boolean;
  projectRoot: string;
  workspace?: V1WorkspaceRef | null;
  teams: V1TeamRef[];
  projects: V1Project[];
  issues: V1Issue[];
  views: V1View[];
  invariants: string[];
  sources: string[];
  boundary: WorkbenchBoundary;
};

export type ProjectFilesSnapshot = {
  version: string;
  projectRoot: string;
  entries: ProjectFileEntry[];
  selectedPath?: string | null;
};

export type ProjectFileEntry = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  childCount?: number | null;
  isSymlink?: boolean;
  children: ProjectFileChild[];
};

export type ProjectFileChild = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  childCount?: number | null;
  isSymlink?: boolean;
};

export type ProjectFileContent = {
  relativePath: string;
  name: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  mimeType?: string | null;
  language: string;
  content?: string | null;
  binaryPreview?: string | null;
  dataUrl?: string | null;
  truncated?: boolean;
  directoryChildren: ProjectFileChild[];
  unsupportedReason?: string | null;
};

export type ProjectFileViewMode = "source" | "all" | "recent";

export type ProjectDirectoryPage = {
  version: string;
  projectRoot: string;
  directoryPath: string;
  entries: ProjectFileChild[];
  nextCursor?: string | null;
  totalChildren: number;
  limit: number;
  viewMode: ProjectFileViewMode | string;
};

export type ProjectFileSearchSnapshot = {
  version: string;
  projectRoot: string;
  query: string;
  viewMode: ProjectFileViewMode | string;
  results: ProjectFileSearchResult[];
};

export type ProjectFileSearchResult = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  extension?: string | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  score: number;
  matchReason: string;
};

export type ProjectFileTextRange = {
  version: string;
  projectRoot: string;
  relativePath: string;
  startLine: number;
  endLine: number;
  totalLines: number;
  content: string;
  truncated: boolean;
};

export type ProjectRecommendedFile = {
  path: string;
  name: string;
  source: "context-pack-file" | "context-pack-test" | "manifest-important";
  reason: string;
  status: "available" | "missing" | "unloaded";
};

export type GraphStatus = "missing" | "indexing" | "ready" | "stale" | "failed" | "degraded";

export type GraphStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: GraphStatus;
  fileCount: number;
  symbolCount: number;
  relationCount: number;
  updatedAt?: number | null;
  lastError?: string | null;
  watcherStatus?: string | null;
  watcherBackend?: string | null;
  preflightStatus?: string | null;
  protectionStatus?: string | null;
  degradedReasons?: string[];
};

export type GraphManifestSnapshot = {
  version: string;
  projectRoot: string;
  languages: string[];
  topLevelDirs: string[];
  importantFiles: string[];
  sourceFiles: number;
  testFiles: number;
  docFiles: number;
  configFiles: number;
  platforms?: string[];
  entryPoints?: string[];
  mobileComponents?: string[];
  mobileConfigs?: string[];
  mobileTests?: string[];
};

export type GraphSearchSnapshot = {
  version: string;
  query: string;
  results: GraphSearchResult[];
};

export type GraphSearchResult = {
  kind: string;
  path: string;
  title: string;
  language?: string | null;
  symbolKind?: string | null;
  line?: number | null;
  snippet?: string | null;
  score: number;
};

export type GraphContextPack = {
  version: string;
  targetType: string;
  targetId?: string | null;
  query: string;
  createdAt: number;
  graphRevision?: string | null;
  recommendedFiles: GraphContextFile[];
  recommendedSymbols: GraphContextSymbol[];
  recommendedTests: GraphContextFile[];
  impactHints: GraphContextHint[];
  testHints: GraphTestHint[];
  confidence: string;
};

export type GraphContextFile = {
  path: string;
  reason: string;
  score: number;
};

export type GraphContextSymbol = {
  name: string;
  kind: string;
  path: string;
  line: number;
  score: number;
};

export type GraphContextHint = {
  path: string;
  reason: string;
  confidence: string;
};

export type GraphTestHint = {
  commandHint: string;
  reason: string;
  confidence: string;
  scope?: string;
};

export type V1WorkspaceRef = {
  id: string;
  name: string;
  activeProjectId: string;
  teamIds: string[];
  projectIds: string[];
};

export type V1TeamRef = {
  id: string;
  name: string;
  projectIds: string[];
  issueIds: string[];
};

export type V1Project = {
  id: string;
  name: string;
  status: string;
  rawStatus: string;
  goal: string;
  targetMaturity?: string | null;
  targetLayers: string[];
  scope: string[];
  nonGoals: string[];
  successCriteria: string[];
  milestones: V1Milestone[];
  issueOrder: string[];
  validationGate: string[];
  evidenceRequired: string[];
  queueRule: string[];
  closureGate: string[];
};

export type V1Milestone = {
  id: string;
  projectId: string;
  name: string;
  status: string;
  rawStatus: string;
  goal: string;
  entryCriteria: string[];
  scope: string[];
  nonGoals: string[];
  issueIds: string[];
  exitCriteria: string[];
  validation: string[];
  evidenceRequired: string[];
  nextMilestoneGate: string;
  progress: MilestoneDerivedProgress;
};

export type V1Issue = {
  id: string;
  projectId?: string | null;
  milestoneId?: string | null;
  title: string;
  status: string;
  rawStatus: string;
  goal: string;
  scope: string[];
  nonGoals: string[];
  dependencies: string[];
  codexInstructions: string[];
  acceptanceCriteria: string[];
  validationCommands: string[];
  evidenceRequired: string[];
  allowedFiles: string[];
  forbiddenFiles: string[];
  boundary: string[];
  riskLevel: string;
};

export type V1View = {
  id: string;
  name: string;
  entity: string;
  filter: V1ViewFilter;
  sort: V1ViewSort[];
  layout: string;
};

export type V1ViewFilter = {
  issueStatus?: string | null;
  runStatus?: string | null;
  validationStatus?: string | null;
  issueId?: string | null;
};

export type V1ViewSort = {
  field: string;
  direction: string;
};

export type LocalSearchSnapshot = {
  version: string;
  initialized: boolean;
  projectRoot: string;
  query: LocalSearchQuery;
  results: LocalSearchResult[];
  searchedPaths: string[];
  excludedPaths: string[];
  boundary: WorkbenchBoundary;
};

export type LocalSearchQuery = {
  query: string;
};

export type LocalSearchResult = {
  sourceType: string;
  entityKind: string;
  entityId?: string | null;
  path: string;
  title: string;
  field: string;
  line: number;
  snippet: string;
  score: number;
};
