import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewAgentEnvironmentStatus,
} from "../../../browserPreviewData";
import type { AgentEnvironmentStatus } from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type AgentManualState = {
  status: AgentEnvironmentStatus | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialAgentManualState: AgentManualState = {
  status: null,
  error: null,
  source: "idle",
};

export function useAgentManual(projectRoot: string | null) {
  const [agentManualState, setAgentManualState] = useState<AgentManualState>(initialAgentManualState);

  const loadAgentManual = useCallback(
    async (root = projectRoot) => {
      if (!root) {
        setAgentManualState(initialAgentManualState);
        return;
      }

      if (isBrowserPreviewRuntime()) {
        setAgentManualState({
          status: createBrowserPreviewAgentEnvironmentStatus(root ?? BROWSER_PREVIEW_PROJECT_ROOT),
          error: null,
          source: "preview",
        });
        return;
      }

      setAgentManualState((current) => ({ ...current, error: null, source: "loading" }));
      try {
        const status = await invoke<AgentEnvironmentStatus>("prepare_agent_working_manual", {
          projectRoot: root,
        });
        setAgentManualState({ status, error: null, source: "tauri" });
      } catch (error) {
        setAgentManualState({
          status: null,
          error: error instanceof Error ? error.message : String(error),
          source: "unavailable",
        });
      }
    },
    [projectRoot],
  );

  useEffect(() => {
    void loadAgentManual(projectRoot);
  }, [loadAgentManual, projectRoot]);

  return {
    agentManualState,
    loadAgentManual,
  };
}
