type MetricCardProps = {
  detail?: string;
  label: string;
  value: number | string;
};

export function MetricCard({ detail, label, value }: MetricCardProps) {
  return (
    <article className="af-metric-card" data-agentflow-component="metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
      {detail ? <small>{detail}</small> : null}
    </article>
  );
}
