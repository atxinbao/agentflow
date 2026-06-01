import "./AgentStatusBar.css";
import type { AgentStatusChannelItem } from "./statusTypes";

export function AgentStatusBar({ items }: { items: AgentStatusChannelItem[] }) {
  const visibleItems = items.length
    ? items
    : [
        {
          id: "agent-status-empty",
          label: "Agent 状态通道",
          status: "idle" as const,
          statusLabel: "等待数据",
        },
      ];

  return (
    <footer className="shell-status-bar" aria-label="Agent 状态通道">
      {visibleItems.map((item) => (
        <section className="agent-status-item" key={item.id} title={item.source}>
          <div className="agent-status-primary">
            <span className="agent-status-indicator" data-status={item.status} />
            <strong>{item.label}</strong>
            <span>{item.statusLabel}</span>
          </div>
          {item.metrics?.length ? (
            <dl className="agent-status-metrics">
              {item.metrics.map((metric) => (
                <div key={`${item.id}-${metric.label}`}>
                  <dt>{metric.label}</dt>
                  <dd title={metric.title ?? String(metric.value)}>{metric.value}</dd>
                </div>
              ))}
            </dl>
          ) : null}
          {item.error ? <span className="agent-status-error">{item.error}</span> : null}
        </section>
      ))}
    </footer>
  );
}
