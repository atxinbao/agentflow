import type { CSSProperties } from "react";
import {
  ActionButton,
  AdvancedDetailsDrawer,
  BlockedState,
  Button,
  CopyableCodeBlock,
  EmptyState,
  ListPanel,
  ListRow,
  LoadingState,
  MetricCard,
  PageHeader,
  Panel,
  ReadOnlyBadge,
  RiskBadge,
  StatusBadge,
  WarningState,
} from "../../components";
import "./DesignSystemPreview.css";

const colorSwatches = [
  ["背景", "--af-bg", "var(--af-bg)"],
  ["表面", "--af-surface", "var(--af-surface)"],
  ["弱表面", "--af-surface-muted", "var(--af-surface-muted)"],
  ["边框", "--af-border", "var(--af-border)"],
  ["强边框", "--af-border-strong", "var(--af-border-strong)"],
  ["主色", "--af-accent", "var(--af-accent)"],
  ["成功", "--af-success", "var(--af-success)"],
  ["警告", "--af-warning", "var(--af-warning)"],
  ["阻断", "--af-danger", "var(--af-danger)"],
] as const;

const handoffExample = `请在 AgentFlow Desktop 中使用 design system 组件。
范围：apps/desktop/src/styles/**、apps/desktop/src/design/** 和 apps/desktop/src/components/**。
不要改 Rust 后端，不新增 Tauri command，不写 .agentflow。`;

type SwatchStyle = CSSProperties & {
  "--af-swatch-color": string;
};

export function DesignSystemPreview() {
  return (
    <section className="af-design-system-preview af-ui" data-agentflow-design-system="v1" aria-label="Design System Preview">
      <header className="af-preview-header">
        <p className="af-kicker">Figma SVG Style Foundation V1</p>
        <h2 className="af-title">AgentFlow 基础组件</h2>
        <p className="af-description">先统一 token、窗口外壳、面板、列表和状态，再逐页迁移业务页面。</p>
      </header>

      <section className="af-preview-section" aria-label="颜色">
        <h3 className="af-title">颜色</h3>
        <div className="af-preview-swatches">
          {colorSwatches.map(([label, token, color]) => (
            <span className="af-swatch" key={token} style={{ "--af-swatch-color": color } as SwatchStyle}>
              <strong>{label}</strong>
              <code>{token}</code>
            </span>
          ))}
        </div>
      </section>

      <section className="af-preview-section" aria-label="按钮和状态">
        <h3 className="af-title">按钮和状态</h3>
        <div className="af-preview-row">
          <Button variant="primary">告诉 Agent 你想做什么</Button>
          <Button variant="secondary">查看详情</Button>
          <Button variant="ghost">返回</Button>
          <Button variant="danger">返工</Button>
          <ActionButton loading>正在准备</ActionButton>
        </div>
        <div className="af-preview-row">
          <StatusBadge status="ready">Ready</StatusBadge>
          <StatusBadge status="working">Working</StatusBadge>
          <StatusBadge status="warning">Warning</StatusBadge>
          <StatusBadge status="blocked">Blocked</StatusBadge>
          <StatusBadge status="failed">Failed</StatusBadge>
          <StatusBadge status="idle">Idle</StatusBadge>
          <RiskBadge risk="high" />
          <ReadOnlyBadge />
        </div>
      </section>

      <section className="af-preview-section" aria-label="页面基础组件">
        <PageHeader
          description="页面标题只说明当前工作面，不做营销说明。"
          kicker="Project Home"
          meta={<ReadOnlyBadge>本地只读</ReadOnlyBadge>}
          title="项目工作台"
        />
        <div className="af-preview-grid">
          <Panel
            title="可以进入执行了"
            description="这个任务已经有 approved SPEC 和 Issue。"
            footer={<ActionButton variant="primary">复制执行指令</ActionButton>}
          >
            <StatusBadge status="ready">Ready</StatusBadge>
          </Panel>
          <ListPanel count={2} title="任务列表">
            <ListRow
              meta={<StatusBadge status="ready">Ready</StatusBadge>}
              subtitle="低 · 2 条验证命令"
              title="实现 Figma SVG Style Foundation"
            />
            <ListRow
              meta={<StatusBadge status="warning">Review</StatusBadge>}
              subtitle="high · 1 条验证命令"
              title="核对交付证据"
            />
          </ListPanel>
        </div>
      </section>

      <section className="af-preview-section" aria-label="卡片和指标">
        <h3 className="af-title">卡片和指标</h3>
        <div className="af-preview-grid">
          <Panel
            title="项目已准备好"
            description="AgentFlow 已经准备好规则和项目现场。下一步先把需求说清楚。"
            footer={<ActionButton variant="primary">继续整理 SPEC</ActionButton>}
          >
            <StatusBadge status="ready">工作手册：已就绪</StatusBadge>
          </Panel>
          <Panel title="可以审计交付结果" description="交付材料已有 evidence / delivery 摘要。" tone="success">
            <div className="af-preview-metrics">
              <MetricCard label="证据" value={3} detail="可审计" />
              <MetricCard label="交付" value={1} detail="已回填" />
              <MetricCard label="审计" value={0} detail="未请求" />
              <MetricCard label="未完成" value={0} detail="无阻断" />
            </div>
          </Panel>
        </div>
      </section>

      <section className="af-preview-section" aria-label="状态组件">
        <h3 className="af-title">空态 / 阻断态 / Loading / Warning</h3>
        <div className="af-preview-grid">
          <EmptyState
            title="还没有需求。"
            description="先告诉 Spec Agent 你想做什么。"
            primaryAction={<Button variant="primary">开始整理</Button>}
          />
          <BlockedState
            title="还不能进入执行。"
            reason="这个需求还没有确认成 SPEC。"
            action={<Button variant="secondary">继续整理 SPEC</Button>}
          />
          <LoadingState title="正在准备项目现场" />
          <WarningState description="可以继续，但 Git 状态暂时无法确认。" title="项目现场有风险" />
        </div>
      </section>

      <section className="af-preview-section" aria-label="代码块和高级详情">
        <h3 className="af-title">代码块和高级详情</h3>
        <div className="af-preview-grid">
          <CopyableCodeBlock content={handoffExample} language="md" title="执行指令" />
          <AdvancedDetailsDrawer description="默认隐藏内部细节" title="高级详情">
            <p>这里可以放 workflow gates、blockers、audit index 和 report refs。普通用户默认不用看。</p>
          </AdvancedDetailsDrawer>
        </div>
      </section>
    </section>
  );
}
