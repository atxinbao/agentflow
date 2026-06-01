export type AgentStatusTone = "idle" | "working" | "ready" | "warning" | "failed";

export type AgentStatusMetric = {
  label: string;
  value: string | number;
  title?: string;
};

export type AgentStatusChannelItem = {
  id: string;
  label: string;
  status: AgentStatusTone;
  statusLabel: string;
  source?: string;
  priority?: number;
  metrics?: AgentStatusMetric[];
  error?: string | null;
};
