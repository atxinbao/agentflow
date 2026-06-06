import type { ButtonHTMLAttributes, ComponentProps, HTMLAttributes, ReactNode } from "react";
import { Button } from "../Button";
import type { StatusChipStatus } from "../StatusChip";

type Tone = "neutral" | "success" | "warning" | "blocked" | "danger";
type RiskLevel = "low" | "normal" | "medium" | "high" | "critical" | string;

function cx(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(" ");
}

export function AppFrame({ children, className, ...props }: HTMLAttributes<HTMLElement>) {
  return (
    <main className={cx("af-app-frame", className)} data-agentflow-component="app-frame" {...props}>
      {children}
    </main>
  );
}

type WindowChromeProps = HTMLAttributes<HTMLElement> & {
  actions?: ReactNode;
  description?: ReactNode;
  kicker?: ReactNode;
  title?: ReactNode;
};

export function WindowChrome({
  actions,
  children,
  className,
  description,
  kicker,
  title,
  ...props
}: WindowChromeProps) {
  return (
    <section className={cx("af-window-chrome", className)} data-agentflow-component="window-chrome" {...props}>
      {title || description || kicker || actions ? (
        <header className="af-window-chrome-header">
          {kicker ? <p className="af-window-chrome-kicker">{kicker}</p> : null}
          {title ? <h1 className="af-window-chrome-title">{title}</h1> : null}
          {description ? <p className="af-window-chrome-description">{description}</p> : null}
          {actions ? <div className="af-window-chrome-actions">{actions}</div> : null}
        </header>
      ) : null}
      {children}
    </section>
  );
}

type TopBarProps = HTMLAttributes<HTMLElement> & {
  actions?: ReactNode;
  leading?: ReactNode;
  meta?: ReactNode;
  search?: ReactNode;
  subtitle?: ReactNode;
  title?: ReactNode;
};

export function TopBar({ actions, children, className, leading, meta, search, subtitle, title, ...props }: TopBarProps) {
  if (children) {
    return (
      <header className={cx("af-top-bar", className)} data-agentflow-component="top-bar" {...props}>
        {children}
      </header>
    );
  }

  return (
    <header className={cx("af-top-bar", className)} data-agentflow-component="top-bar" {...props}>
      <div className="af-top-bar-leading">
        {leading}
        <div className="af-top-bar-title">
          {title ? <strong>{title}</strong> : null}
          {subtitle ? <span>{subtitle}</span> : null}
        </div>
      </div>
      {search}
      <div className="af-top-bar-actions">
        {actions}
        {meta}
      </div>
    </header>
  );
}

export function Sidebar({ children, className, ...props }: HTMLAttributes<HTMLElement>) {
  return (
    <aside className={cx("af-sidebar", className)} data-agentflow-component="sidebar" {...props}>
      {children}
    </aside>
  );
}

type PageHeaderProps = HTMLAttributes<HTMLElement> & {
  actions?: ReactNode;
  description?: ReactNode;
  kicker?: ReactNode;
  meta?: ReactNode;
  title: ReactNode;
};

export function PageHeader({ actions, className, description, kicker, meta, title, ...props }: PageHeaderProps) {
  return (
    <header className={cx("af-page-header", className)} data-agentflow-component="page-header" {...props}>
      <div className="af-page-header-main">
        {kicker ? <p className="af-page-kicker">{kicker}</p> : null}
        <h2 className="af-page-title">{title}</h2>
        {description ? <p className="af-page-description">{description}</p> : null}
      </div>
      {actions || meta ? (
        <div className="af-page-header-aside">
          {meta}
          {actions}
        </div>
      ) : null}
    </header>
  );
}

type PanelProps = HTMLAttributes<HTMLElement> & {
  actions?: ReactNode;
  description?: ReactNode;
  footer?: ReactNode;
  kicker?: ReactNode;
  title?: ReactNode;
  tone?: Tone;
};

export function Panel({
  actions,
  children,
  className,
  description,
  footer,
  kicker,
  title,
  tone = "neutral",
  ...props
}: PanelProps) {
  return (
    <section
      className={cx("af-panel", tone !== "neutral" && `af-panel-${tone}`, className)}
      data-agentflow-component="panel"
      {...props}
    >
      {title || description || kicker || actions ? (
        <header className="af-panel-header">
          {kicker ? <p className="af-panel-kicker">{kicker}</p> : null}
          {title ? <h3 className="af-panel-title">{title}</h3> : null}
          {description ? <p className="af-panel-description">{description}</p> : null}
          {actions ? <div className="af-panel-actions">{actions}</div> : null}
        </header>
      ) : null}
      {children}
      {footer ? <footer className="af-panel-footer">{footer}</footer> : null}
    </section>
  );
}

type SectionProps = HTMLAttributes<HTMLElement> & {
  description?: ReactNode;
  title?: ReactNode;
};

export function Section({ children, className, description, title, ...props }: SectionProps) {
  return (
    <section className={cx("af-section", className)} data-agentflow-component="section" {...props}>
      {title || description ? (
        <header className="af-section-header">
          {title ? <h3 className="af-section-title">{title}</h3> : null}
          {description ? <p className="af-section-description">{description}</p> : null}
        </header>
      ) : null}
      {children}
    </section>
  );
}

type ListPanelProps = HTMLAttributes<HTMLElement> & {
  count?: ReactNode;
  empty?: ReactNode;
  title: ReactNode;
};

export function ListPanel({ children, className, count, empty, title, ...props }: ListPanelProps) {
  return (
    <section className={cx("af-list-panel", className)} data-agentflow-component="list-panel" {...props}>
      <header className="af-list-panel-header">
        <h3 className="af-list-panel-title">{title}</h3>
        {count !== undefined ? <span className="af-list-panel-count">{count}</span> : null}
      </header>
      <div className="af-list-panel-body">{children || empty}</div>
    </section>
  );
}

type ListRowProps = Omit<HTMLAttributes<HTMLElement>, "title"> & {
  active?: boolean;
  footer?: ReactNode;
  meta?: ReactNode;
  onClick?: ButtonHTMLAttributes<HTMLButtonElement>["onClick"];
  subtitle?: ReactNode;
  title: ReactNode;
};

export function ListRow({ active = false, className, footer, meta, onClick, subtitle, title, ...props }: ListRowProps) {
  const classes = cx("af-list-row", active && "af-list-row-active", className);
  const content = (
    <>
      {meta ? <div className="af-list-row-meta">{meta}</div> : null}
      <strong className="af-list-row-title">{title}</strong>
      {subtitle ? <span className="af-list-row-subtitle">{subtitle}</span> : null}
      {footer ? <small className="af-list-row-footer">{footer}</small> : null}
    </>
  );

  if (onClick) {
    return (
      <button className={classes} data-agentflow-component="list-row" onClick={onClick} type="button">
        {content}
      </button>
    );
  }

  return (
    <article className={classes} data-agentflow-component="list-row" {...props}>
      {content}
    </article>
  );
}

type StatusBadgeProps = HTMLAttributes<HTMLSpanElement> & {
  children?: ReactNode;
  status?: StatusChipStatus;
};

export function StatusBadge({ children, className, status = "idle", ...props }: StatusBadgeProps) {
  return (
    <span
      className={cx("af-status-badge", `af-status-badge-${status}`, className)}
      data-agentflow-component="status-badge"
      {...props}
    >
      {children ?? status}
    </span>
  );
}

export function RiskBadge({
  "aria-label": ariaLabel,
  className,
  risk = "normal",
  ...props
}: HTMLAttributes<HTMLSpanElement> & { risk?: RiskLevel }) {
  const normalized = String(risk || "normal").toLowerCase();
  const level = normalized.includes("critical")
    ? "critical"
    : normalized.includes("high")
      ? "high"
      : normalized.includes("medium")
        ? "medium"
        : normalized.includes("low")
          ? "low"
          : "normal";
  const label =
    level === "critical"
      ? "严重"
      : level === "high"
        ? "高"
        : level === "medium"
          ? "中"
          : level === "low"
            ? "低"
            : "普通";
  return (
    <span
      aria-label={ariaLabel ?? `风险：${label}`}
      className={cx("af-risk-badge", `af-risk-badge-${level}`, className)}
      data-agentflow-component="risk-badge"
      {...props}
    />
  );
}

export function ReadOnlyBadge({ children = "只读", className, ...props }: HTMLAttributes<HTMLSpanElement>) {
  return (
    <span className={cx("af-readonly-badge", className)} data-agentflow-component="readonly-badge" {...props}>
      {children}
    </span>
  );
}

export function ActionButton({ className, ...props }: ComponentProps<typeof Button>) {
  return <Button className={cx("af-action-button", className)} data-agentflow-component="action-button" {...props} />;
}

export function StatusBar({ children, className, ...props }: HTMLAttributes<HTMLElement>) {
  return (
    <footer className={cx("af-status-bar", className)} data-agentflow-component="status-bar" {...props}>
      {children}
    </footer>
  );
}
