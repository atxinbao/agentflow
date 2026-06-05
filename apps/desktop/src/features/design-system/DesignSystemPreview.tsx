import type { CSSProperties } from "react";
import {
  AdvancedDetailsDrawer,
  BlockedState,
  Button,
  CopyableCodeBlock,
  EmptyState,
  LoadingState,
  MetricCard,
  StatusChip,
  SurfaceCard,
  WarningState,
} from "../../components";
import "./DesignSystemPreview.css";

const colorSwatches = [
  ["背景", "--af-bg", "#0b1020"],
  ["表面", "--af-surface", "#11182b"],
  ["表面 2", "--af-surface-2", "#151e33"],
  ["主色", "--af-primary", "#7c9cff"],
  ["成功", "--af-success", "#22c55e"],
  ["警告", "--af-warning", "#fbbf24"],
  ["阻断", "--af-danger", "#fb7185"],
  ["代码", "--af-blue", "#38bdf8"],
] as const;

const handoffExample = `请在 AgentFlow Desktop 中使用 design system 组件。
范围：apps/desktop/src/design/** 和 apps/desktop/src/components/**。
不要改 Rust 后端，不新增 Tauri command，不写 .agentflow。`;

type SwatchStyle = CSSProperties & {
  "--af-swatch-color": string;
};

export function DesignSystemPreview() {
  return (
    <section className="af-design-system-preview af-ui" data-agentflow-design-system="v1" aria-label="Design System Preview">
      <header className="af-preview-header">
        <p className="af-kicker">Desktop Design System V1</p>
        <h2 className="af-title">AgentFlow 基础组件</h2>
        <p className="af-description">先把颜色、按钮、卡片和状态统一，再做 Project Home 和三 Agent 动线。</p>
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
          <Button loading>正在准备</Button>
        </div>
        <div className="af-preview-row">
          <StatusChip status="ready" />
          <StatusChip status="working" />
          <StatusChip status="warning" />
          <StatusChip status="blocked" />
          <StatusChip status="failed" />
          <StatusChip status="idle" />
        </div>
      </section>

      <section className="af-preview-section" aria-label="卡片和指标">
        <h3 className="af-title">卡片和指标</h3>
        <div className="af-preview-grid">
          <SurfaceCard
            title="项目已准备好"
            description="AgentFlow 已经准备好规则和项目现场。下一步先把需求说清楚。"
            footer={<Button variant="primary">继续整理 SPEC</Button>}
          >
            <StatusChip status="ready">工作手册：已就绪</StatusChip>
          </SurfaceCard>
          <SurfaceCard title="可以交给 Codex 了" description="这个任务已经有 approved SPEC 和 Issue。" tone="success">
            <div className="af-preview-metrics">
              <MetricCard label="证据" value={3} detail="可审计" />
              <MetricCard label="交付" value={1} detail="已回填" />
              <MetricCard label="审计" value={0} detail="未请求" />
              <MetricCard label="未完成" value={0} detail="无阻断" />
            </div>
          </SurfaceCard>
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
            title="还不能交给 Codex。"
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
          <CopyableCodeBlock content={handoffExample} language="md" title="Codex 指令" />
          <AdvancedDetailsDrawer description="默认隐藏内部细节" title="高级详情">
            <p>这里可以放 workflow gates、blockers、audit index 和 report refs。普通用户默认不用看。</p>
          </AdvancedDetailsDrawer>
        </div>
      </section>
    </section>
  );
}
