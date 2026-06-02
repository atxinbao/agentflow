import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { createBrowserPreviewGoalTreeSnapshot } from "../../../browserPreviewData";
import type { GoalTreeSnapshot } from "../../../types";
import { isBrowserPreviewRuntime, normalizeProjectRootKey } from "../../project-files";
import { defaultGoalTreeSelection, type GoalTreeSelection } from "../model/goalTreeUtils";

export type GoalTreeState = {
  snapshot: GoalTreeSnapshot | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  source: "loading" | "tauri" | "preview" | "unavailable";
};

export function useGoalTree(projectRoot: string | null) {
  const [state, setState] = useState<GoalTreeState>({
    snapshot: null,
    loading: false,
    saving: false,
    error: null,
    source: "loading",
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

  useEffect(() => {
    void loadGoalTree();
  }, [loadGoalTree]);

  return {
    loadGoalTree,
    selection,
    setSelection,
    state,
  };
}

function readableError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
