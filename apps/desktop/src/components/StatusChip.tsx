import type { HTMLAttributes, ReactNode } from "react";

export type StatusChipStatus = "ready" | "working" | "warning" | "blocked" | "failed" | "idle";

const statusLabels: Record<StatusChipStatus, string> = {
  blocked: "已阻断",
  failed: "异常",
  idle: "未开始",
  ready: "已就绪",
  warning: "有风险",
  working: "准备中",
};

type StatusChipProps = HTMLAttributes<HTMLSpanElement> & {
  children?: ReactNode;
  status?: StatusChipStatus;
};

export function StatusChip({ children, className, status = "idle", ...props }: StatusChipProps) {
  return (
    <span
      className={["af-status-chip", `af-status-${status}`, className].filter(Boolean).join(" ")}
      data-agentflow-component="status-chip"
      {...props}
    >
      {children ?? statusLabels[status]}
    </span>
  );
}
