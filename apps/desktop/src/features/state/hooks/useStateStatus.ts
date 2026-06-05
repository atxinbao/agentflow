import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewIssueStatusIndex,
  createBrowserPreviewStateStatus,
} from "../../../browserPreviewData";
import type { IssueStatusIndex, StateStatusSnapshot } from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type StateStatusState = {
  status: StateStatusSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialStateStatusState: StateStatusState = {
  status: null,
  error: null,
  source: "idle",
};

export function useStateStatus(projectRoot: string | null) {
  const [stateStatusState, setStateStatusState] = useState<StateStatusState>(initialStateStatusState);

  useEffect(() => {
    if (!projectRoot) {
      setStateStatusState(initialStateStatusState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setStateStatusState({
        status: createBrowserPreviewStateStatus(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setStateStatusState((current) => ({ ...current, error: null, source: "loading" }));
    void invoke<StateStatusSnapshot>("load_state_status", { projectRoot })
      .then((status) => {
        if (!cancelled) {
          setStateStatusState({ status, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setStateStatusState({
            status: null,
            error: error instanceof Error ? error.message : String(error),
            source: "unavailable",
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot]);

  return stateStatusState;
}

export type IssueStatusIndexState = {
  index: IssueStatusIndex | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialIssueStatusIndexState: IssueStatusIndexState = {
  index: null,
  error: null,
  source: "idle",
};

export function useIssueStatusIndex(projectRoot: string | null, refreshToken = 0) {
  const [issueStatusIndexState, setIssueStatusIndexState] =
    useState<IssueStatusIndexState>(initialIssueStatusIndexState);

  useEffect(() => {
    if (!projectRoot) {
      setIssueStatusIndexState(initialIssueStatusIndexState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setIssueStatusIndexState({
        index: createBrowserPreviewIssueStatusIndex(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setIssueStatusIndexState((current) => ({ ...current, error: null, source: "loading" }));
    void invoke<IssueStatusIndex>("load_issue_status_index", { projectRoot })
      .then((index) => {
        if (!cancelled) {
          setIssueStatusIndexState({ index, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setIssueStatusIndexState({
            index: null,
            error: error instanceof Error ? error.message : String(error),
            source: "unavailable",
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, refreshToken]);

  return issueStatusIndexState;
}
