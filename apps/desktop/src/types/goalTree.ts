export type GoalTreeSnapshot = {
  version: string;
  projectRoot: string;
  index: GoalTreeIndex;
  goals: GoalRecord[];
  milestones: MilestoneRecord[];
  issues: IssueRecord[];
  validation: GoalTreeValidationSnapshot;
};

export type GoalTreeIndex = {
  version: string;
  projectRoot: string;
  activeGoalId?: string | null;
  goalOrder: string[];
  milestoneOrderByGoal: Record<string, string[]>;
  issueOrderByMilestone: Record<string, string[]>;
  updatedAt: number;
};

export type GoalRecord = {
  version: "goal.v1" | string;
  id: string;
  projectRoot: string;
  status: GoalStatus | string;
  human: GoalHumanContract;
  agentDraft: GoalAgentDraft;
  system: GoalSystemState;
};

export type MilestoneRecord = {
  version: "milestone.v1" | string;
  id: string;
  goalId: string;
  projectRoot: string;
  status: MilestoneStatus | string;
  human: MilestoneHumanContract;
  agentDraft: MilestoneAgentDraft;
  system: MilestoneSystemState;
};

export type IssueRecord = {
  version: "issue.v1" | string;
  id: string;
  goalId: string;
  milestoneId: string;
  projectRoot: string;
  status: IssueStatus | string;
  human: IssueHumanContract;
  agentDraft: IssueAgentDraft;
  system: IssueSystemState;
};

export type GoalStatus = "draft" | "active" | "paused" | "completed" | "archived";
export type MilestoneStatus = "draft" | "planned" | "active" | "blocked" | "completed" | "archived";
export type IssueStatus = "draft" | "ready" | "blocked" | "completed" | "canceled" | "archived";

export type GoalHumanContract = {
  title: string;
  objective: string;
  scope: string[];
  nonGoals: string[];
  successCriteria: string[];
  milestoneOrder: string[];
  validationGate: string[];
  closureGate: string[];
};

export type MilestoneHumanContract = {
  title: string;
  stageGoal: string;
  entryCriteria: string[];
  scope: string[];
  nonGoals: string[];
  issueOrder: string[];
  exitCriteria: string[];
  nextGate: string[];
};

export type IssueHumanContract = {
  title: string;
  goal: string;
  scope: string[];
  nonGoals: string[];
  dependencies: string[];
  acceptanceCriteria: string[];
  validationCommands: string[];
  evidenceRequirements: string[];
  boundary: string[];
};

export type GoalAgentDraft = {
  suggestedMilestones: string[];
  suggestedRisks: string[];
  suggestedQuestions: string[];
  suggestedIssueBreakdown: string[];
};

export type MilestoneAgentDraft = {
  suggestedIssues: string[];
  suggestedRisks: string[];
  suggestedQuestions: string[];
};

export type IssueAgentDraft = {
  suggestedFiles: string[];
  suggestedSymbols: string[];
  suggestedTests: string[];
  suggestedImplementationPlan: string[];
  suggestedRisks: string[];
  questions: string[];
};

export type GoalSystemState = {
  createdAt: number;
  updatedAt: number;
  createdBy: string;
  updatedBy: string;
  path: string;
  revision: number;
};

export type MilestoneSystemState = GoalSystemState;

export type IssueSystemState = GoalSystemState & {
  graphContextPackPath?: string | null;
};

export type GoalTreeValidationSnapshot = {
  version: string;
  projectRoot: string;
  valid: boolean;
  errors: GoalTreeValidationIssue[];
  warnings: GoalTreeValidationIssue[];
};

export type GoalTreeValidationIssue = {
  code: string;
  message: string;
  objectType: "goal" | "milestone" | "issue" | string;
  objectId?: string | null;
};

export type GoalTreeIssueContextSnapshot = {
  version: string;
  projectRoot: string;
  issueId: string;
  status: string;
  contextPackPath?: string | null;
  recommendedFiles: GoalTreeRecommendedFile[];
  recommendedTests: GoalTreeRecommendedFile[];
  warnings: string[];
};

export type GoalTreeRecommendedFile = {
  path: string;
  reason: string;
  score: number;
};

export type CreateGoalInput = {
  status?: GoalStatus;
  title: string;
  objective: string;
  scope?: string[];
  nonGoals?: string[];
  successCriteria?: string[];
  validationGate?: string[];
  closureGate?: string[];
};

export type CreateMilestoneInput = {
  status?: MilestoneStatus;
  title: string;
  stageGoal: string;
  entryCriteria?: string[];
  scope?: string[];
  nonGoals?: string[];
  exitCriteria?: string[];
  nextGate?: string[];
};

export type CreateIssueInput = {
  status?: IssueStatus;
  title: string;
  goal: string;
  scope?: string[];
  nonGoals?: string[];
  dependencies?: string[];
  acceptanceCriteria?: string[];
  validationCommands?: string[];
  evidenceRequirements?: string[];
  boundary?: string[];
};
