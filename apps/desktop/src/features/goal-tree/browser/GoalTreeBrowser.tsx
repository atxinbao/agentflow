import { ChevronDown, ChevronRight, CircleDot, Flag, ListChecks } from "lucide-react";
import type { GoalTreeSnapshot } from "../../../types";
import {
  orderedGoals,
  orderedIssuesForMilestone,
  orderedMilestonesForGoal,
  type GoalTreeSelection,
} from "../model/goalTreeUtils";

export function GoalTreeBrowser({
  onSelect,
  selection,
  snapshot,
}: {
  onSelect: (selection: GoalTreeSelection) => void;
  selection: GoalTreeSelection | null;
  snapshot: GoalTreeSnapshot;
}) {
  const goals = orderedGoals(snapshot).filter((goal) => goal.status !== "archived");
  return (
    <aside className="goal-tree-browser" aria-label="Goal Tree 列表">
      <header>
        <div>
          <span className="goal-tree-kicker">Goal Tree</span>
          <h2>目标树</h2>
        </div>
        <span className={snapshot.validation.valid ? "goal-tree-health valid" : "goal-tree-health invalid"}>
          {snapshot.validation.valid ? "完整" : "需处理"}
        </span>
      </header>

      {goals.length === 0 ? (
        <p className="goal-tree-empty">暂无 Goal。先创建一个项目目标。</p>
      ) : (
        <div className="goal-tree-outline">
          {goals.map((goal) => (
            <section key={goal.id} className="goal-tree-node-group">
              <button
                className={selection?.type === "goal" && selection.id === goal.id ? "goal-tree-row active" : "goal-tree-row"}
                onClick={() => onSelect({ type: "goal", id: goal.id })}
                type="button"
              >
                <ChevronDown size={14} />
                <CircleDot size={14} />
                <span>{goal.human.title || goal.id}</span>
                <small>{goal.status}</small>
              </button>
              <div className="goal-tree-children">
                {orderedMilestonesForGoal(snapshot, goal.id).map((milestone) => (
                  <section key={milestone.id}>
                    <button
                      className={
                        selection?.type === "milestone" && selection.id === milestone.id
                          ? "goal-tree-row milestone active"
                          : "goal-tree-row milestone"
                      }
                      onClick={() => onSelect({ type: "milestone", id: milestone.id })}
                      type="button"
                    >
                      <ChevronDown size={14} />
                      <Flag size={14} />
                      <span>{milestone.human.title || milestone.id}</span>
                      <small>{milestone.status}</small>
                    </button>
                    <div className="goal-tree-children nested">
                      {orderedIssuesForMilestone(snapshot, milestone.id).map((issue) => (
                        <button
                          className={
                            selection?.type === "issue" && selection.id === issue.id
                              ? "goal-tree-row issue active"
                              : "goal-tree-row issue"
                          }
                          key={issue.id}
                          onClick={() => onSelect({ type: "issue", id: issue.id })}
                          type="button"
                        >
                          <ChevronRight size={14} />
                          <ListChecks size={14} />
                          <span>{issue.human.title || issue.id}</span>
                          <small>{issue.status}</small>
                        </button>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
            </section>
          ))}
        </div>
      )}
    </aside>
  );
}
