import type { ReactNode } from "react";

type BlockedStateProps = {
  action?: ReactNode;
  children?: ReactNode;
  reason: ReactNode;
  technicalDetails?: ReactNode;
  title: ReactNode;
};

export function BlockedState({ action, children, reason, technicalDetails, title }: BlockedStateProps) {
  return (
    <section className="af-state af-state-blocked" data-agentflow-component="blocked-state">
      <header className="af-surface-card-header">
        <h3 className="af-title">{title}</h3>
        <p className="af-blocked-reason">
          <strong>原因：</strong>
          <span>{reason}</span>
        </p>
      </header>
      {children}
      {action ? <div className="af-state-actions">{action}</div> : null}
      {technicalDetails ? <div>{technicalDetails}</div> : null}
    </section>
  );
}
