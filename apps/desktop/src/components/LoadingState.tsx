type LoadingStateProps = {
  steps?: string[];
  title: string;
};

const defaultSteps = ["正在准备 Agent 工作手册", "正在读取项目现场", "正在生成工作流状态"];

export function LoadingState({ steps = defaultSteps, title }: LoadingStateProps) {
  return (
    <section
      aria-busy="true"
      aria-live="polite"
      className="af-state af-state-loading"
      data-agentflow-component="loading-state"
    >
      <header className="af-surface-card-header">
        <h3 className="af-title">{title}</h3>
        <p className="af-description">请等当前步骤完成。</p>
      </header>
      <ul className="af-loading-steps">
        {steps.map((step) => (
          <li key={step}>{step}</li>
        ))}
      </ul>
    </section>
  );
}
