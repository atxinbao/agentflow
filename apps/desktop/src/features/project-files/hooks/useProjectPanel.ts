import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import {
  createBrowserPreviewPanelContextPack,
  createBrowserPreviewPanelManifest,
  createBrowserPreviewPanelSearch,
  createBrowserPreviewPanelStatus,
} from "../../../browserPreviewData";
import type {
  PanelContextPack,
  PanelManifestSnapshot,
  PanelSearchSnapshot,
  PanelStatusSnapshot,
} from "../../../types";
import { isBrowserPreviewRuntime } from "./projectFileRuntime";

export type ProjectPanelState = {
  status: PanelStatusSnapshot | null;
  manifest: PanelManifestSnapshot | null;
  latestSearch: PanelSearchSnapshot | null;
  latestContextPack: PanelContextPack | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialProjectPanelState: ProjectPanelState = {
  status: null,
  manifest: null,
  latestSearch: null,
  latestContextPack: null,
  error: null,
  source: "idle",
};

function readableProjectPanelError(error: unknown) {
  if (isBrowserPreviewRuntime()) {
    return "当前为浏览器预览，正在显示 Mock 项目现场；真实 Panel 请在桌面客户端查看。";
  }
  return error instanceof Error ? error.message : String(error);
}

export function useProjectPanel(projectRoot: string | null) {
  const [projectPanelState, setProjectPanelState] = useState<ProjectPanelState>(initialProjectPanelState);

  const loadProjectPanel = useCallback(
    async (root = projectRoot) => {
      if (!root) {
        setProjectPanelState(initialProjectPanelState);
        return;
      }
      setProjectPanelState((current) => ({ ...current, error: null, source: "loading" }));
      try {
        const [status, manifest] = await Promise.all([
          invoke<PanelStatusSnapshot>("load_project_panel_status", { projectRoot: root }),
          invoke<PanelManifestSnapshot>("load_project_panel_manifest", { projectRoot: root }),
        ]);
        setProjectPanelState((current) => ({
          ...current,
          status,
          manifest,
          error: null,
          source: "tauri",
        }));
        void invoke("dispatch_workflow_events", { projectRoot: root }).catch(() => undefined);
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          setProjectPanelState((current) => ({
            ...current,
            status: createBrowserPreviewPanelStatus(root),
            manifest: createBrowserPreviewPanelManifest(root),
            error: null,
            source: "preview",
          }));
          return;
        }
        setProjectPanelState((current) => ({
          ...current,
          status: null,
          manifest: null,
          error: readableProjectPanelError(error),
          source: "unavailable",
        }));
      }
    },
    [projectRoot],
  );

  const prepareProjectPanel = useCallback(
    async (root = projectRoot) => {
      if (!root) {
        setProjectPanelState(initialProjectPanelState);
        return;
      }
      setProjectPanelState((current) => ({ ...current, error: null, source: "loading" }));
      try {
        const status = await invoke<PanelStatusSnapshot>("prepare_project_panel", { projectRoot: root });
        const manifest = await invoke<PanelManifestSnapshot>("load_project_panel_manifest", { projectRoot: root });
        setProjectPanelState((current) => ({
          ...current,
          status,
          manifest,
          error: null,
          source: "tauri",
        }));
        void invoke("dispatch_workflow_events", { projectRoot: root }).catch(() => undefined);
        if (status.status === "indexing") {
          window.setTimeout(() => {
            void loadProjectPanel(root);
          }, 900);
        }
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          setProjectPanelState((current) => ({
            ...current,
            status: createBrowserPreviewPanelStatus(root),
            manifest: createBrowserPreviewPanelManifest(root),
            error: null,
            source: "preview",
          }));
          return;
        }
        setProjectPanelState((current) => ({
          ...current,
          status: null,
          manifest: null,
          error: readableProjectPanelError(error),
          source: "unavailable",
        }));
      }
    },
    [loadProjectPanel, projectRoot],
  );

  const searchProjectPanel = useCallback(
    async (query: string, limit = 20, root = projectRoot) => {
      if (!root) {
        return null;
      }
      try {
        const result = await invoke<PanelSearchSnapshot>("search_project_panel", {
          projectRoot: root,
          query,
          limit,
        });
        setProjectPanelState((current) => ({ ...current, latestSearch: result, error: null }));
        return result;
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const result = createBrowserPreviewPanelSearch(query);
          setProjectPanelState((current) => ({ ...current, latestSearch: result, error: null, source: "preview" }));
          return result;
        }
        setProjectPanelState((current) => ({ ...current, error: readableProjectPanelError(error) }));
        return null;
      }
    },
    [projectRoot],
  );

  const buildPanelContextPack = useCallback(
    async (
      targetType: string,
      title: string,
      objective: string,
      acceptanceCriteria: string[],
      targetId?: string | null,
      root = projectRoot,
    ) => {
      if (!root) {
        return null;
      }
      try {
        const pack = await invoke<PanelContextPack>("build_panel_context_pack", {
          projectRoot: root,
          targetType,
          targetId,
          title,
          objective,
          acceptanceCriteria,
        });
        setProjectPanelState((current) => ({ ...current, latestContextPack: pack, error: null }));
        return pack;
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const pack = createBrowserPreviewPanelContextPack(root);
          setProjectPanelState((current) => ({
            ...current,
            latestContextPack: pack,
            error: null,
            source: "preview",
          }));
          return pack;
        }
        setProjectPanelState((current) => ({ ...current, error: readableProjectPanelError(error) }));
        return null;
      }
    },
    [projectRoot],
  );

  useEffect(() => {
    void prepareProjectPanel(projectRoot);
  }, [prepareProjectPanel, projectRoot]);

  return {
    buildPanelContextPack,
    loadProjectPanel,
    prepareProjectPanel,
    projectPanelState,
    searchProjectPanel,
  };
}
