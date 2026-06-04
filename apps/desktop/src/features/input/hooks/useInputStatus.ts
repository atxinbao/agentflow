import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewInputStatus,
} from "../../../browserPreviewData";
import type { InputStatusSnapshot } from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type InputStatusState = {
  status: InputStatusSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialInputStatusState: InputStatusState = {
  status: null,
  error: null,
  source: "idle",
};

export function useInputStatus(projectRoot: string | null) {
  const [inputStatusState, setInputStatusState] = useState<InputStatusState>(initialInputStatusState);

  useEffect(() => {
    if (!projectRoot) {
      setInputStatusState(initialInputStatusState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setInputStatusState({
        status: createBrowserPreviewInputStatus(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setInputStatusState((current) => ({ ...current, error: null, source: "loading" }));
    void invoke<InputStatusSnapshot>("load_input_status", { projectRoot })
      .then((status) => {
        if (!cancelled) {
          setInputStatusState({ status, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setInputStatusState({
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

  return inputStatusState;
}
