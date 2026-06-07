import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewInputSnapshot,
  createBrowserPreviewInputStatus,
} from "../../../browserPreviewData";
import type { InputSnapshot, InputStatusSnapshot } from "../../../types";
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

function inputErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function loadInputStatusWithRepair(projectRoot: string) {
  try {
    return await invoke<InputStatusSnapshot>("load_input_status", { projectRoot });
  } catch (loadError) {
    try {
      const snapshot = await invoke<InputSnapshot>("prepare_input_workspace", { projectRoot });
      return snapshot.status;
    } catch (repairError) {
      throw new Error(
        `load input status failed: ${inputErrorMessage(loadError)}; repair failed: ${inputErrorMessage(repairError)}`,
      );
    }
  }
}

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
    void loadInputStatusWithRepair(projectRoot)
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

export type InputSnapshotState = {
  snapshot: InputSnapshot | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialInputSnapshotState: InputSnapshotState = {
  snapshot: null,
  error: null,
  source: "idle",
};

async function loadInputSnapshotWithRepair(projectRoot: string) {
  try {
    return await invoke<InputSnapshot>("load_input_snapshot", { projectRoot });
  } catch (loadError) {
    try {
      return await invoke<InputSnapshot>("prepare_input_workspace", { projectRoot });
    } catch (repairError) {
      throw new Error(
        `load input snapshot failed: ${inputErrorMessage(loadError)}; repair failed: ${inputErrorMessage(repairError)}`,
      );
    }
  }
}

export function useInputSnapshot(projectRoot: string | null) {
  const [inputSnapshotState, setInputSnapshotState] =
    useState<InputSnapshotState>(initialInputSnapshotState);

  useEffect(() => {
    if (!projectRoot) {
      setInputSnapshotState(initialInputSnapshotState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setInputSnapshotState({
        snapshot: createBrowserPreviewInputSnapshot(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setInputSnapshotState((current) => ({ ...current, error: null, source: "loading" }));
    void loadInputSnapshotWithRepair(projectRoot)
      .then((snapshot) => {
        if (!cancelled) {
          setInputSnapshotState({ snapshot, error: null, source: "tauri" });
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setInputSnapshotState({
            snapshot: null,
            error: error instanceof Error ? error.message : String(error),
            source: "unavailable",
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [projectRoot]);

  return inputSnapshotState;
}
