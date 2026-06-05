import type { ReactNode } from "react";

type EmptyStateProps = {
  description: ReactNode;
  primaryAction?: ReactNode;
  secondaryAction?: ReactNode;
  title: ReactNode;
};

export function EmptyState({ description, primaryAction, secondaryAction, title }: EmptyStateProps) {
  return (
    <section className="af-state af-state-empty" data-agentflow-component="empty-state">
      <header className="af-surface-card-header">
        <h3 className="af-title">{title}</h3>
        <p className="af-description">{description}</p>
      </header>
      {primaryAction || secondaryAction ? (
        <div className="af-state-actions">
          {primaryAction}
          {secondaryAction}
        </div>
      ) : null}
    </section>
  );
}
