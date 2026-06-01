import "./AgentStatusBar.css";
import type { AgentStatusChannelItem } from "./statusTypes";

export function AgentStatusBar({ items }: { items: AgentStatusChannelItem[] }) {
  const visibleItem = selectVisibleStatusItem(items);

  return (
    <footer className="shell-status-bar" aria-label="Agent 状态通道">
      <section className="agent-status-item" title={buildStatusTitle(visibleItem)}>
        <div className="agent-status-primary">
          <span className="agent-status-indicator" data-status={visibleItem.status} />
          <strong>{visibleItem.label}</strong>
          <span className="agent-status-separator" aria-hidden="true">
            {" · "}
          </span>
          <span>{visibleItem.statusLabel}</span>
        </div>
      </section>
    </footer>
  );
}

function selectVisibleStatusItem(items: AgentStatusChannelItem[]) {
  if (!items.length) {
    return {
      id: "agent-status-empty",
      label: "状态通道",
      status: "idle" as const,
      statusLabel: "等待数据",
    };
  }

  return [...items].sort((left, right) => {
    const toneDiff = statusToneWeight(right.status) - statusToneWeight(left.status);
    if (toneDiff !== 0) {
      return toneDiff;
    }
    return (right.priority ?? 0) - (left.priority ?? 0);
  })[0];
}

function statusToneWeight(status: AgentStatusChannelItem["status"]) {
  const weights: Record<AgentStatusChannelItem["status"], number> = {
    failed: 50,
    warning: 40,
    working: 30,
    ready: 20,
    idle: 10,
  };
  return weights[status];
}

function buildStatusTitle(item: AgentStatusChannelItem) {
  const detailLines = [
    item.source ? `来源：${item.source}` : null,
    `状态：${item.statusLabel}`,
    ...(item.metrics ?? []).map((metric) => {
      const value = metric.title ?? String(metric.value);
      return `${metric.label}：${value}`;
    }),
    item.error ? `错误：${item.error}` : null,
  ].filter(Boolean);

  return detailLines.join("\n");
}
