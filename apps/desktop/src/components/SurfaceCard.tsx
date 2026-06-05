import type { HTMLAttributes, ReactNode } from "react";

type SurfaceCardTone = "neutral" | "success" | "warning" | "blocked" | "danger";

type SurfaceCardProps = HTMLAttributes<HTMLElement> & {
  compact?: boolean;
  description?: ReactNode;
  footer?: ReactNode;
  title?: ReactNode;
  tone?: SurfaceCardTone;
};

export function SurfaceCard({
  children,
  className,
  compact = false,
  description,
  footer,
  title,
  tone = "neutral",
  ...props
}: SurfaceCardProps) {
  const classes = [
    "af-surface-card",
    compact ? "af-surface-card-compact" : "",
    tone !== "neutral" ? `af-surface-card-${tone}` : "",
    className,
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <article className={classes} data-agentflow-component="surface-card" {...props}>
      {title || description ? (
        <header className="af-surface-card-header">
          {title ? <h3 className="af-title">{title}</h3> : null}
          {description ? <p className="af-description">{description}</p> : null}
        </header>
      ) : null}
      {children}
      {footer ? <footer className="af-surface-card-footer">{footer}</footer> : null}
    </article>
  );
}
