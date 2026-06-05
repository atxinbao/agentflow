import type { ReactNode } from "react";

type AdvancedDetailsDrawerProps = {
  children: ReactNode;
  defaultOpen?: boolean;
  description?: ReactNode;
  title: ReactNode;
};

export function AdvancedDetailsDrawer({ children, defaultOpen = false, description, title }: AdvancedDetailsDrawerProps) {
  return (
    <details className="af-advanced-details" data-agentflow-component="advanced-details-drawer" open={defaultOpen}>
      <summary>
        <span>{title}</span>
        {description ? <small>{description}</small> : null}
      </summary>
      <div className="af-advanced-details-body">{children}</div>
    </details>
  );
}
