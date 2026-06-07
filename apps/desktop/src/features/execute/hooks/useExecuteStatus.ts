import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewExecuteStatus,
} from "../../../browserPreviewData";
import type { ExecuteStatusSnapshot } from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type ExecuteStatusState = {
  status: ExecuteStatusSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialExecuteStatusState: ExecuteStatusState = {
  status: null,
  error: null,
  source: "idle",
};

export function useExecuteStatus(projectRoot: string | null, refreshToken = 0) {
  const [executeStatusState, setExecuteStatusState] = useState<ExecuteStatusState>(initialExecuteStatusState);

  useEffect(() => {
    if (!projectRoot) {
      setExecuteStatusState(initialExecuteStatusState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setExecuteStatusState({
        status: createBrowserPreviewExecuteStatus(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setExecuteStatusState((current) =>
      current.status ? { ...current, error: null } : { ...current, error: null, source: "loading" },
    );
    void invoke<ExecuteStatusSnapshot>("load_execute_status", { projectRoot })
      .then((status) => {
        if (!cancelled) {
          setExecuteStatusState({ status, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : String(error);
          setExecuteStatusState((current) =>
            current.status
              ? { ...current, error: message }
              : {
                  status: null,
                  error: message,
                  source: "unavailable",
                },
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot, refreshToken]);

  return executeStatusState;
}
