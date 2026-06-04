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
