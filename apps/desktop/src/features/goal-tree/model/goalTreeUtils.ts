import type { GoalRecord, GoalTreeSnapshot, IssueRecord, MilestoneRecord } from "../../../types";

export type GoalTreeSelection =
  | { type: "goal"; id: string }
  | { type: "milestone"; id: string }
  | { type: "issue"; id: string };

export function defaultGoalTreeSelection(snapshot: GoalTreeSnapshot | null): GoalTreeSelection | null {
  const activeGoal = snapshot?.goals.find((goal) => goal.id === snapshot.index.activeGoalId) ?? snapshot?.goals.at(0);
  return activeGoal ? { type: "goal", id: activeGoal.id } : null;
}

export function findSelectedGoalTreeRecord(snapshot: GoalTreeSnapshot | null, selection: GoalTreeSelection | null) {
  if (!snapshot || !selection) {
    return null;
  }
  if (selection.type === "goal") {
    return snapshot.goals.find((goal) => goal.id === selection.id) ?? null;
  }
  if (selection.type === "milestone") {
    return snapshot.milestones.find((milestone) => milestone.id === selection.id) ?? null;
  }
  return snapshot.issues.find((issue) => issue.id === selection.id) ?? null;
}

export function orderedGoals(snapshot: GoalTreeSnapshot): GoalRecord[] {
  return orderedByIds(snapshot.goals, snapshot.index.goalOrder);
}

export function orderedMilestonesForGoal(snapshot: GoalTreeSnapshot, goalId: string): MilestoneRecord[] {
  const milestones = snapshot.milestones.filter((milestone) => milestone.goalId === goalId && milestone.status !== "archived");
  return orderedByIds(milestones, snapshot.index.milestoneOrderByGoal[goalId] ?? []);
}

export function orderedIssuesForMilestone(snapshot: GoalTreeSnapshot, milestoneId: string): IssueRecord[] {
  const issues = snapshot.issues.filter((issue) => issue.milestoneId === milestoneId && issue.status !== "archived");
  return orderedByIds(issues, snapshot.index.issueOrderByMilestone[milestoneId] ?? []);
}

export function linesToList(value: string): string[] {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

export function listToLines(values: string[]): string {
  return values.join("\n");
}

function orderedByIds<T extends { id: string }>(records: T[], orderedIds: string[]): T[] {
  const index = new Map(orderedIds.map((id, position) => [id, position]));
  return [...records].sort((left, right) => {
    const leftIndex = index.get(left.id) ?? Number.MAX_SAFE_INTEGER;
    const rightIndex = index.get(right.id) ?? Number.MAX_SAFE_INTEGER;
    if (leftIndex !== rightIndex) {
      return leftIndex - rightIndex;
    }
    return left.id.localeCompare(right.id);
  });
}
