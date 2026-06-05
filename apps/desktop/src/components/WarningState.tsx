import type { ReactNode } from "react";

type WarningStateProps = {
  action?: ReactNode;
  description: ReactNode;
  title: ReactNode;
};

export function WarningState({ action, description, title }: WarningStateProps) {
  return (
    <section className="af-state af-state-warning" data-agentflow-component="warning-state">
      <header className="af-surface-card-header">
        <h3 className="af-title">{title}</h3>
        <p className="af-description">{description}</p>
      </header>
      {action ? <div className="af-state-actions">{action}</div> : null}
    </section>
  );
}
