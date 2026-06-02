import { RefreshCw } from "lucide-react";
import { useMemo } from "react";
import type { GoalRecord, GoalTreeSnapshot, IssueRecord, MilestoneRecord } from "../../types";
import { GoalTreeBrowser } from "./browser/GoalTreeBrowser";
import { GoalTreeContextPanel } from "./context/GoalTreeContextPanel";
import { GoalContractViewer, IssueContractViewer, MilestoneContractViewer } from "./editor/GoalEditor";
import "./GoalTree.css";
import type { useGoalTree } from "./hooks/useGoalTree";
import {
  defaultGoalTreeSelection,
  findSelectedGoalTreeRecord,
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
  const { loadGoalTree, selection, setSelection, state } = controller;
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
          <span className="goal-tree-readonly-pill">只读</span>
          <span className="goal-tree-agent-pill">Agent-only</span>
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
            <GoalTreeViewerSwitch
              selection={selection ?? defaultGoalTreeSelection(snapshot)}
              snapshot={snapshot}
            />
          </main>
          <GoalTreeContextPanel
            issue={selectedIssue}
            onOpenFile={onOpenProjectFile}
            snapshot={snapshot}
          />
        </div>
      ) : (
        <section className="goal-tree-empty-state">
          <h2>还没有 Agent 准备的 Goal Tree</h2>
          <p>Goal Tree 将由后续 Agent planning flow 写入。Desktop 只负责读取和审查，不创建、不编辑、不写入。</p>
        </section>
      )}
    </section>
  );
}

function GoalTreeViewerSwitch({
  selection,
  snapshot,
}: {
  selection: GoalTreeSelection | null;
  snapshot: GoalTreeSnapshot;
}) {
  const record = findSelectedGoalTreeRecord(snapshot, selection);
  if (!record || !selection) {
    return (
      <section className="goal-tree-empty">
        <h2>等待 Agent 准备目标树</h2>
        <p>当前项目还没有可查看的 Goal / Milestone / Issue。Desktop 不提供手动创建入口。</p>
      </section>
    );
  }
  if (selection.type === "goal") {
    return <GoalContractViewer goal={record as GoalRecord} />;
  }
  if (selection.type === "milestone") {
    return <MilestoneContractViewer milestone={record as MilestoneRecord} />;
  }
  return <IssueContractViewer issue={record as IssueRecord} />;
}
