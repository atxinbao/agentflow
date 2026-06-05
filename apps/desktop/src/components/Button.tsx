import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";
type ButtonSize = "sm" | "md" | "lg";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  leftIcon?: ReactNode;
  loading?: boolean;
  rightIcon?: ReactNode;
  size?: ButtonSize;
  variant?: ButtonVariant;
};

export function Button({
  children,
  className,
  disabled,
  leftIcon,
  loading = false,
  rightIcon,
  size = "md",
  type = "button",
  variant = "secondary",
  ...props
}: ButtonProps) {
  const classes = ["af-button", `af-button-${variant}`, `af-button-${size}`, className].filter(Boolean).join(" ");

  return (
    <button
      aria-busy={loading || undefined}
      className={classes}
      data-agentflow-component="button"
      disabled={disabled || loading}
      type={type}
      {...props}
    >
      {loading ? <span className="af-button-spinner" aria-hidden="true" /> : leftIcon}
      <span>{children}</span>
      {rightIcon}
    </button>
  );
}
