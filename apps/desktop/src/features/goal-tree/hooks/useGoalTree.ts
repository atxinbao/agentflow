import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { createBrowserPreviewGoalTreeSnapshot } from "../../../browserPreviewData";
import type {
  CreateGoalInput,
  CreateIssueInput,
  CreateMilestoneInput,
  GoalRecord,
  GoalTreeIssueContextSnapshot,
  GoalTreeSnapshot,
  IssueRecord,
  MilestoneRecord,
} from "../../../types";
import { isBrowserPreviewRuntime, normalizeProjectRootKey } from "../../project-files";
import { defaultGoalTreeSelection, type GoalTreeSelection } from "../model/goalTreeUtils";

export type GoalTreeState = {
  snapshot: GoalTreeSnapshot | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  source: "loading" | "tauri" | "preview" | "unavailable";
  context: GoalTreeIssueContextSnapshot | null;
  contextLoading: boolean;
};

export function useGoalTree(projectRoot: string | null) {
  const [state, setState] = useState<GoalTreeState>({
    snapshot: null,
    loading: false,
    saving: false,
    error: null,
    source: "loading",
    context: null,
    contextLoading: false,
  });
  const [selection, setSelection] = useState<GoalTreeSelection | null>(null);

  const loadGoalTree = useCallback(
    async (root = projectRoot) => {
      const normalizedRoot = normalizeProjectRootKey(root ?? "");
      if (!normalizedRoot) {
        setState((current) => ({
          ...current,
          snapshot: null,
          loading: false,
          error: "请先选择本地 Project Workspace。",
          source: "unavailable",
        }));
        setSelection(null);
        return;
      }
      setState((current) => ({ ...current, loading: true, error: null }));
      if (isBrowserPreviewRuntime()) {
        const snapshot = createBrowserPreviewGoalTreeSnapshot(normalizedRoot);
        setState((current) => ({ ...current, snapshot, loading: false, source: "preview" }));
        setSelection((current) => current ?? defaultGoalTreeSelection(snapshot));
        return;
      }
      try {
        const snapshot = await invoke<GoalTreeSnapshot>("load_goal_tree_snapshot", { projectRoot: normalizedRoot });
        setState((current) => ({ ...current, snapshot, loading: false, source: "tauri" }));
        setSelection((current) => current ?? defaultGoalTreeSelection(snapshot));
      } catch (error) {
        setState((current) => ({
          ...current,
          loading: false,
          error: readableError(error),
          source: "unavailable",
        }));
      }
    },
    [projectRoot],
  );

  async function mutate<T>(operation: () => Promise<T>) {
    setState((current) => ({ ...current, saving: true, error: null }));
    try {
      const result = await operation();
      await loadGoalTree();
      setState((current) => ({ ...current, saving: false }));
      return result;
    } catch (error) {
      setState((current) => ({ ...current, saving: false, error: readableError(error) }));
      throw error;
    }
  }

  const createGoal = useCallback(
    (input: CreateGoalInput) =>
      mutate(async () => {
        if (isBrowserPreviewRuntime()) {
          throw new Error("浏览器预览不写真实 .agentflow/；请使用桌面客户端创建 Goal。");
        }
        const record = await invoke<GoalRecord>("create_goal_tree_goal", { projectRoot, input });
        setSelection({ type: "goal", id: record.id });
        return record;
      }),
    [projectRoot, loadGoalTree],
  );

  const createMilestone = useCallback(
    (goalId: string, input: CreateMilestoneInput) =>
      mutate(async () => {
        if (isBrowserPreviewRuntime()) {
          throw new Error("浏览器预览不写真实 .agentflow/；请使用桌面客户端创建 Milestone。");
        }
        const record = await invoke<MilestoneRecord>("create_goal_tree_milestone", { projectRoot, goalId, input });
        setSelection({ type: "milestone", id: record.id });
        return record;
      }),
    [projectRoot, loadGoalTree],
  );

  const createIssue = useCallback(
    (milestoneId: string, input: CreateIssueInput) =>
      mutate(async () => {
        if (isBrowserPreviewRuntime()) {
          throw new Error("浏览器预览不写真实 .agentflow/；请使用桌面客户端创建 Issue。");
        }
        const record = await invoke<IssueRecord>("create_goal_tree_issue", { projectRoot, milestoneId, input });
        setSelection({ type: "issue", id: record.id });
        return record;
      }),
    [projectRoot, loadGoalTree],
  );

  const updateGoal = useCallback(
    (goalId: string, patch: Record<string, unknown>) =>
      mutate(() => invoke<GoalRecord>("update_goal_tree_goal", { projectRoot, goalId, patch })),
    [projectRoot, loadGoalTree],
  );

  const updateMilestone = useCallback(
    (milestoneId: string, patch: Record<string, unknown>) =>
      mutate(() => invoke<MilestoneRecord>("update_goal_tree_milestone", { projectRoot, milestoneId, patch })),
    [projectRoot, loadGoalTree],
  );

  const updateIssue = useCallback(
    (issueId: string, patch: Record<string, unknown>) =>
      mutate(() => invoke<IssueRecord>("update_goal_tree_issue", { projectRoot, issueId, patch })),
    [projectRoot, loadGoalTree],
  );

  const archiveSelection = useCallback(() => {
    if (!selection) {
      return Promise.resolve(null);
    }
    if (selection.type === "goal") {
      return mutate(() => invoke("archive_goal_tree_goal", { projectRoot, goalId: selection.id }));
    }
    if (selection.type === "milestone") {
      return mutate(() => invoke("archive_goal_tree_milestone", { projectRoot, milestoneId: selection.id }));
    }
    return mutate(() => invoke("archive_goal_tree_issue", { projectRoot, issueId: selection.id }));
  }, [projectRoot, selection, loadGoalTree]);

  const reorderGoalTree = useCallback(
    (input: Record<string, unknown>) =>
      mutate(() => invoke<GoalTreeSnapshot>("reorder_goal_tree", { projectRoot, input })),
    [projectRoot, loadGoalTree],
  );

  const prepareIssueContext = useCallback(
    async (issueId: string) => {
      if (!projectRoot) {
        return;
      }
      setState((current) => ({ ...current, contextLoading: true, error: null }));
      try {
        const context = isBrowserPreviewRuntime()
          ? null
          : await invoke<GoalTreeIssueContextSnapshot>("prepare_goal_tree_issue_context", {
              projectRoot,
              issueId,
            });
        setState((current) => ({
          ...current,
          context: context ?? current.context,
          contextLoading: false,
        }));
        await loadGoalTree();
      } catch (error) {
        setState((current) => ({ ...current, contextLoading: false, error: readableError(error) }));
      }
    },
    [projectRoot, loadGoalTree],
  );

  useEffect(() => {
    void loadGoalTree();
  }, [loadGoalTree]);

  return {
    archiveSelection,
    createGoal,
    createIssue,
    createMilestone,
    loadGoalTree,
    prepareIssueContext,
    reorderGoalTree,
    selection,
    setSelection,
    state,
    updateGoal,
    updateIssue,
    updateMilestone,
  };
}

function readableError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
