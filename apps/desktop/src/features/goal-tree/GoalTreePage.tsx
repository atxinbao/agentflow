import { Plus, RefreshCw } from "lucide-react";
import { useMemo, useState, type FormEvent } from "react";
import type { GoalRecord, GoalTreeSnapshot, IssueRecord, MilestoneRecord } from "../../types";
import { GoalTreeBrowser } from "./browser/GoalTreeBrowser";
import { GoalTreeContextPanel } from "./context/GoalTreeContextPanel";
import { GoalEditor } from "./editor/GoalEditor";
import { IssueEditor } from "./editor/IssueEditor";
import { MilestoneEditor } from "./editor/MilestoneEditor";
import "./GoalTree.css";
import type { useGoalTree } from "./hooks/useGoalTree";
import {
  defaultGoalTreeSelection,
  findSelectedGoalTreeRecord,
  orderedGoals,
  orderedMilestonesForGoal,
  type GoalTreeSelection,
} from "./model/goalTreeUtils";

type GoalTreeController = ReturnType<typeof useGoalTree>;

export function GoalTreePage({
  controller,
  onOpenProjectFile,
  projectRoot,
}: {
  controller: GoalTreeController;
  onOpenProjectFile: (relativePath: string) => void;
  projectRoot: string | null;
}) {
  const {
    archiveSelection,
    createGoal,
    createIssue,
    createMilestone,
    loadGoalTree,
    prepareIssueContext,
    selection,
    setSelection,
    state,
    updateGoal,
    updateIssue,
    updateMilestone,
  } = controller;
  const [createMode, setCreateMode] = useState<"goal" | "milestone" | "issue" | null>(null);
  const snapshot = state.snapshot;
  const selectedRecord = useMemo(() => findSelectedGoalTreeRecord(snapshot, selection), [selection, snapshot]);
  const selectedIssue = selection?.type === "issue" ? (selectedRecord as IssueRecord | null) : null;

  if (!projectRoot) {
    return (
      <section className="goal-tree-page empty">
        <h2>Goal Tree</h2>
        <p>请先从左侧添加或选择一个本地 Project Workspace。</p>
      </section>
    );
  }

  if (state.loading && !snapshot) {
    return (
      <section className="goal-tree-page empty">
        <RefreshCw className="spin" size={20} />
        <span>正在读取 Goal Tree...</span>
      </section>
    );
  }

  return (
    <section className="goal-tree-page" aria-label="Goal Tree V1">
      <div className="goal-tree-toolbar">
        <div>
          <span className="goal-tree-kicker">Goal Tree V1</span>
          <h1>本地目标树</h1>
          <p>{projectRoot}</p>
        </div>
        <div className="goal-tree-toolbar-actions">
          <button onClick={() => void loadGoalTree()} type="button">
            <RefreshCw size={15} />
            刷新
          </button>
          <button onClick={() => setCreateMode("goal")} type="button">
            <Plus size={15} />
            Goal
          </button>
          <button disabled={!snapshot?.goals.length} onClick={() => setCreateMode("milestone")} type="button">
            <Plus size={15} />
            Milestone
          </button>
          <button disabled={!snapshot?.milestones.length} onClick={() => setCreateMode("issue")} type="button">
            <Plus size={15} />
            Issue
          </button>
        </div>
      </div>

      {state.error ? <div className="goal-tree-error">{state.error}</div> : null}

      {snapshot ? (
        <div className="goal-tree-layout">
          <GoalTreeBrowser
            onSelect={setSelection}
            selection={selection ?? defaultGoalTreeSelection(snapshot)}
            snapshot={snapshot}
          />
          <main className="goal-tree-detail">
            {createMode ? (
              <CreateGoalTreeRecordForm
                mode={createMode}
                onCancel={() => setCreateMode(null)}
                onCreateGoal={(input) => {
                  void createGoal(input).then(() => setCreateMode(null));
                }}
                onCreateIssue={(milestoneId, input) => {
                  void createIssue(milestoneId, input).then(() => setCreateMode(null));
                }}
                onCreateMilestone={(goalId, input) => {
                  void createMilestone(goalId, input).then(() => setCreateMode(null));
                }}
                saving={state.saving}
                snapshot={snapshot}
              />
            ) : (
              <GoalTreeEditorSwitch
                onArchive={() => void archiveSelection()}
                onSaveGoal={(goalId, patch) => void updateGoal(goalId, patch)}
                onSaveIssue={(issueId, patch) => void updateIssue(issueId, patch)}
                onSaveMilestone={(milestoneId, patch) => void updateMilestone(milestoneId, patch)}
                saving={state.saving}
                selection={selection ?? defaultGoalTreeSelection(snapshot)}
                snapshot={snapshot}
              />
            )}
          </main>
          <GoalTreeContextPanel
            context={state.context}
            contextLoading={state.contextLoading}
            issue={selectedIssue}
            onOpenFile={onOpenProjectFile}
            onPrepareContext={(issueId) => void prepareIssueContext(issueId)}
            snapshot={snapshot}
          />
        </div>
      ) : (
        <section className="goal-tree-empty-state">
          <h2>还没有 Goal Tree</h2>
          <p>创建第一个 Goal 后，AgentFlow 会写入 `.agentflow/define/**`。</p>
          <button onClick={() => setCreateMode("goal")} type="button">
            创建 Goal
          </button>
        </section>
      )}
    </section>
  );
}

function GoalTreeEditorSwitch({
  onArchive,
  onSaveGoal,
  onSaveIssue,
  onSaveMilestone,
  saving,
  selection,
  snapshot,
}: {
  onArchive: () => void;
  onSaveGoal: (goalId: string, patch: Record<string, unknown>) => void;
  onSaveIssue: (issueId: string, patch: Record<string, unknown>) => void;
  onSaveMilestone: (milestoneId: string, patch: Record<string, unknown>) => void;
  saving: boolean;
  selection: GoalTreeSelection | null;
  snapshot: GoalTreeSnapshot;
}) {
  const record = findSelectedGoalTreeRecord(snapshot, selection);
  if (!record || !selection) {
    return <p className="goal-tree-empty">请选择或创建一个 Goal。</p>;
  }
  if (selection.type === "goal") {
    return <GoalEditor goal={record as GoalRecord} onArchive={onArchive} onSave={onSaveGoal} saving={saving} />;
  }
  if (selection.type === "milestone") {
    return <MilestoneEditor milestone={record as MilestoneRecord} onArchive={onArchive} onSave={onSaveMilestone} saving={saving} />;
  }
  return <IssueEditor issue={record as IssueRecord} onArchive={onArchive} onSave={onSaveIssue} saving={saving} />;
}

function CreateGoalTreeRecordForm({
  mode,
  onCancel,
  onCreateGoal,
  onCreateIssue,
  onCreateMilestone,
  saving,
  snapshot,
}: {
  mode: "goal" | "milestone" | "issue";
  onCancel: () => void;
  onCreateGoal: Parameters<GoalTreeController["createGoal"]>[0] extends infer T ? (input: T) => void : never;
  onCreateIssue: (milestoneId: string, input: Parameters<GoalTreeController["createIssue"]>[1]) => void;
  onCreateMilestone: (goalId: string, input: Parameters<GoalTreeController["createMilestone"]>[1]) => void;
  saving: boolean;
  snapshot: GoalTreeSnapshot;
}) {
  const firstGoal = orderedGoals(snapshot).at(0);
  const firstMilestone = firstGoal ? orderedMilestonesForGoal(snapshot, firstGoal.id).at(0) : snapshot.milestones.at(0);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [parentId, setParentId] = useState(mode === "issue" ? firstMilestone?.id ?? "" : firstGoal?.id ?? "");

  function submit(event: FormEvent) {
    event.preventDefault();
    if (mode === "goal") {
      onCreateGoal({
        title,
        objective: body,
        scope: [],
        nonGoals: ["不启动 Agent", "不执行项目命令", "不调用模型"],
        successCriteria: [],
        validationGate: [],
        closureGate: [],
      });
      return;
    }
    if (mode === "milestone") {
      onCreateMilestone(parentId, {
        title,
        stageGoal: body,
        entryCriteria: [],
        scope: [],
        nonGoals: [],
        exitCriteria: [],
        nextGate: [],
      });
      return;
    }
    onCreateIssue(parentId, {
      title,
      goal: body,
      scope: [],
      nonGoals: [],
      dependencies: [],
      acceptanceCriteria: [],
      validationCommands: [],
      evidenceRequirements: [],
      boundary: ["不启动 Agent", "不执行项目命令", "不写用户源码"],
    });
  }

  return (
    <form className="goal-tree-create-form" onSubmit={submit}>
      <header>
        <span className="goal-tree-kicker">Create</span>
        <h2>创建 {mode}</h2>
      </header>
      {mode !== "goal" ? (
        <label className="goal-tree-field">
          <span>{mode === "milestone" ? "所属 Goal" : "所属 Milestone"}</span>
          <select onChange={(event) => setParentId(event.target.value)} value={parentId}>
            {(mode === "milestone" ? snapshot.goals : snapshot.milestones).map((record) => (
              <option key={record.id} value={record.id}>
                {record.human.title || record.id}
              </option>
            ))}
          </select>
        </label>
      ) : null}
      <label className="goal-tree-field">
        <span>标题</span>
        <input onChange={(event) => setTitle(event.target.value)} required value={title} />
      </label>
      <label className="goal-tree-field">
        <span>{mode === "goal" ? "目标" : mode === "milestone" ? "阶段目标" : "Issue 目标"}</span>
        <textarea onChange={(event) => setBody(event.target.value)} required rows={5} value={body} />
      </label>
      <div className="goal-tree-form-actions">
        <button disabled={saving} type="submit">
          {saving ? "创建中..." : "创建"}
        </button>
        <button onClick={onCancel} type="button">
          取消
        </button>
      </div>
    </form>
  );
}
