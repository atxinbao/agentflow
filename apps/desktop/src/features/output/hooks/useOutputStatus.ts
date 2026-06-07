import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewOutputStatus,
} from "../../../browserPreviewData";
import type { OutputStatusSnapshot } from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type OutputStatusState = {
  status: OutputStatusSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialOutputStatusState: OutputStatusState = {
  status: null,
  error: null,
  source: "idle",
};

export function useOutputStatus(projectRoot: string | null, refreshToken = 0) {
  const [outputStatusState, setOutputStatusState] = useState<OutputStatusState>(initialOutputStatusState);

  useEffect(() => {
    if (!projectRoot) {
      setOutputStatusState(initialOutputStatusState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setOutputStatusState({
        status: createBrowserPreviewOutputStatus(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setOutputStatusState((current) =>
      current.status ? { ...current, error: null } : { ...current, error: null, source: "loading" },
    );
    void invoke<OutputStatusSnapshot>("load_output_status", { projectRoot })
      .then((status) => {
        if (!cancelled) {
          setOutputStatusState({ status, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : String(error);
          setOutputStatusState((current) =>
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

  return outputStatusState;
}
