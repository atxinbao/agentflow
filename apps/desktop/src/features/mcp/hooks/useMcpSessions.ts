import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewMcpSessions,
  currentBrowserPreviewTaskHierarchyScenario,
} from "../../../browserPreviewData";
import type { McpSessionSnapshot } from "../../../types";
import { isBrowserPreviewRuntime } from "../../project-files";

export type McpSessionsState = {
  sessions: McpSessionSnapshot[];
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialMcpSessionsState: McpSessionsState = {
  sessions: [],
  error: null,
  source: "idle",
};

const ACTIVE_SESSION_STATUSES = new Set(["queued", "claimed", "starting", "running", "in-review"]);

function hasActiveSessions(sessions: McpSessionSnapshot[]) {
  return sessions.some((session) => ACTIVE_SESSION_STATUSES.has(session.status));
}

export function useMcpSessions(projectRoot: string | null, refreshToken = 0) {
  const [mcpSessionsState, setMcpSessionsState] = useState<McpSessionsState>(initialMcpSessionsState);

  useEffect(() => {
    if (!projectRoot) {
      setMcpSessionsState(initialMcpSessionsState);
      return;
    }

    if (isBrowserPreviewRuntime()) {
      setMcpSessionsState({
        sessions: createBrowserPreviewMcpSessions(
          projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT,
          currentBrowserPreviewTaskHierarchyScenario(),
        ),
        error: null,
        source: "preview",
      });
      return;
    }

    let cancelled = false;
    setMcpSessionsState((current) =>
      current.sessions.length ? { ...current, error: null } : { ...current, error: null, source: "loading" },
    );

    let pollTimer: number | null = null;

    const pollSessions = async () => {
      try {
        const sessions = await invoke<McpSessionSnapshot[]>("poll_mcp_session_snapshots", { projectRoot });
        if (cancelled) {
          return;
        }
        setMcpSessionsState({ sessions, error: null, source: "tauri" });
        if (hasActiveSessions(sessions)) {
          pollTimer = window.setTimeout(() => {
            void pollSessions();
          }, 1800);
        }
      } catch (error) {
        if (cancelled) {
          return;
        }
        const message = error instanceof Error ? error.message : String(error);
        setMcpSessionsState((current) =>
          current.sessions.length
            ? { ...current, error: message }
            : {
                sessions: [],
                error: message,
                source: "unavailable",
              },
        );
      }
    };

    void pollSessions();

    return () => {
      cancelled = true;
      if (pollTimer) {
        window.clearTimeout(pollTimer);
      }
    };
  }, [projectRoot, refreshToken]);

  return mcpSessionsState;
}
