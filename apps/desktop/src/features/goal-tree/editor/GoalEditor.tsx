import type { ReactNode } from "react";
import type { GoalRecord, IssueRecord, MilestoneRecord } from "../../../types";

export function GoalContractViewer({ goal }: { goal: GoalRecord }) {
  return (
    <ContractViewerShell id={goal.id} kind="Goal" status={goal.status} title={goal.human.title}>
      <ReadOnlySection title="目标约束">
        <ReadOnlyText label="目标" value={goal.human.objective} />
        <ReadOnlyList label="范围" values={goal.human.scope} />
        <ReadOnlyList label="非目标" values={goal.human.nonGoals} />
        <ReadOnlyList label="成功标准" values={goal.human.successCriteria} />
        <ReadOnlyList label="里程碑顺序" values={goal.human.milestoneOrder} />
        <ReadOnlyList label="验证门槛" values={goal.human.validationGate} code />
        <ReadOnlyList label="完成判定" values={goal.human.closureGate} />
      </ReadOnlySection>
      <AgentDraftSection
        items={[
          ["建议里程碑", goal.agentDraft.suggestedMilestones],
          ["建议风险", goal.agentDraft.suggestedRisks],
          ["待确认问题", goal.agentDraft.suggestedQuestions],
          ["建议任务拆分", goal.agentDraft.suggestedIssueBreakdown],
        ]}
      />
      <SystemStateSection
        values={[
          ["创建者", goal.system.createdBy],
          ["更新者", goal.system.updatedBy],
          ["记录路径", goal.system.path],
          ["修订", String(goal.system.revision)],
          ["更新时间", formatTimestamp(goal.system.updatedAt)],
        ]}
      />
    </ContractViewerShell>
  );
}

export function MilestoneContractViewer({ milestone }: { milestone: MilestoneRecord }) {
  return (
    <ContractViewerShell id={milestone.id} kind="Milestone" status={milestone.status} title={milestone.human.title}>
      <ReadOnlySection title="目标约束">
        <ReadOnlyText label="阶段目标" value={milestone.human.stageGoal} />
        <ReadOnlyList label="进入标准" values={milestone.human.entryCriteria} />
        <ReadOnlyList label="范围" values={milestone.human.scope} />
        <ReadOnlyList label="非目标" values={milestone.human.nonGoals} />
        <ReadOnlyList label="Issue 顺序" values={milestone.human.issueOrder} />
        <ReadOnlyList label="退出标准" values={milestone.human.exitCriteria} />
        <ReadOnlyList label="下一阶段门" values={milestone.human.nextGate} />
      </ReadOnlySection>
      <AgentDraftSection
        items={[
          ["建议 Issue", milestone.agentDraft.suggestedIssues],
          ["建议风险", milestone.agentDraft.suggestedRisks],
          ["待确认问题", milestone.agentDraft.suggestedQuestions],
        ]}
      />
      <SystemStateSection
        values={[
          ["所属 Goal", milestone.goalId],
          ["创建者", milestone.system.createdBy],
          ["更新者", milestone.system.updatedBy],
          ["记录路径", milestone.system.path],
          ["修订", String(milestone.system.revision)],
          ["更新时间", formatTimestamp(milestone.system.updatedAt)],
        ]}
      />
    </ContractViewerShell>
  );
}

export function IssueContractViewer({ issue }: { issue: IssueRecord }) {
  return (
    <ContractViewerShell id={issue.id} kind="Issue" status={issue.status} title={issue.human.title}>
      <ReadOnlySection title="目标约束">
        <ReadOnlyText label="目标" value={issue.human.goal} />
        <ReadOnlyList label="范围" values={issue.human.scope} />
        <ReadOnlyList label="非目标" values={issue.human.nonGoals} />
        <ReadOnlyList label="依赖" values={issue.human.dependencies} />
        <ReadOnlyList label="验收标准" values={issue.human.acceptanceCriteria} />
        <ReadOnlyList label="验证命令（只记录，不执行）" values={issue.human.validationCommands} code />
        <ReadOnlyList label="证据要求" values={issue.human.evidenceRequirements} />
        <ReadOnlyList label="边界" values={issue.human.boundary} />
      </ReadOnlySection>
      <AgentDraftSection
        items={[
          ["建议文件", issue.agentDraft.suggestedFiles],
          ["建议符号", issue.agentDraft.suggestedSymbols],
          ["建议测试", issue.agentDraft.suggestedTests],
          ["建议实现计划", issue.agentDraft.suggestedImplementationPlan],
          ["建议风险", issue.agentDraft.suggestedRisks],
          ["待确认问题", issue.agentDraft.questions],
        ]}
      />
      <SystemStateSection
        values={[
          ["所属 Goal", issue.goalId],
          ["所属 Milestone", issue.milestoneId],
          ["Panel Context", issue.system.panelContextPackPath ?? "未准备"],
          ["创建者", issue.system.createdBy],
          ["更新者", issue.system.updatedBy],
          ["记录路径", issue.system.path],
          ["修订", String(issue.system.revision)],
          ["更新时间", formatTimestamp(issue.system.updatedAt)],
        ]}
      />
    </ContractViewerShell>
  );
}

function ContractViewerShell({
  children,
  id,
  kind,
  status,
  title,
}: {
  children: ReactNode;
  id: string;
  kind: string;
  status: string;
  title: string;
}) {
  return (
    <article className="goal-tree-contract-viewer">
      <header className="goal-tree-editor-header">
        <div>
          <span className="goal-tree-kicker">{kind} / {id}</span>
          <h2>{title || "未命名"}</h2>
          <p>Confirmed Contract：由人类确认过的目标约束，供 Agent 使用。Desktop 仅展示，不编辑。</p>
        </div>
        <div className="goal-tree-editor-actions">
          <span className="goal-tree-status-pill">{status}</span>
          <span className="goal-tree-readonly-pill">只读</span>
        </div>
      </header>
      <div className="goal-tree-contract-stack">{children}</div>
    </article>
  );
}

function ReadOnlySection({ children, title }: { children: ReactNode; title: string }) {
  return (
    <section className="goal-tree-readonly-section">
      <h3>{title}</h3>
      {children}
    </section>
  );
}

function ReadOnlyText({ label, value }: { label: string; value: string }) {
  return (
    <div className="goal-tree-readonly-block">
      <span>{label}</span>
      <p>{value || "未记录"}</p>
    </div>
  );
}

function ReadOnlyList({ code = false, label, values }: { code?: boolean; label: string; values: string[] }) {
  return (
    <div className="goal-tree-readonly-block">
      <span>{label}</span>
      {values.length === 0 ? (
        <p>未记录</p>
      ) : (
        <ul className={code ? "goal-tree-code-list" : undefined}>
          {values.map((value) => (
            <li key={value}>{value}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

function AgentDraftSection({ items }: { items: Array<[string, string[]]> }) {
  return (
    <ReadOnlySection title="Agent Draft">
      {items.map(([label, values]) => (
        <ReadOnlyList key={label} label={label} values={values} />
      ))}
    </ReadOnlySection>
  );
}

function SystemStateSection({ values }: { values: Array<[string, string]> }) {
  return (
    <ReadOnlySection title="System State">
      <dl className="goal-tree-system-state">
        {values.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </ReadOnlySection>
  );
}

function formatTimestamp(value: number) {
  if (!value) {
    return "未记录";
  }
  return new Date(value * 1000).toLocaleString();
}
