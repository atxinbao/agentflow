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
};
