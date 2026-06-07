import type { WorkbenchBoundary } from "./workbench";
import type { AgentRole, ExpectedOutputs, IssueCategory, IssueDisplayStatus } from "./status";

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
  issueCategory?: IssueCategory;
  requiredAgentRole?: AgentRole;
  sourceSpecId?: string | null;
  sourceSpecPath?: string | null;
  issuePath?: string | null;
  createdAt?: number | null;
  updatedAt?: number | null;
  handoffId?: string | null;
  contextPackPath?: string | null;
  auditTrigger?: string | null;
  auditId?: string | null;
  sourceReleaseId?: string | null;
  sourceDeliveryPath?: string | null;
  auditOutputDir?: string | null;
  displayStatus?: IssueDisplayStatus;
  status: string;
  rawStatus: string;
  goal: string;
  scope: string[];
  nonGoals: string[];
  dependencies: string[];
  codexInstructions: string[];
  acceptanceCriteria: string[];
  validationCommands: string[];
  expectedOutputs: ExpectedOutputs;
  evidenceRequired: string[];
  allowedFiles: string[];
  forbiddenFiles: string[];
  forbiddenActions: string[];
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
