import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import {
  createBrowserPreviewGraphContextPack,
  createBrowserPreviewGraphManifest,
  createBrowserPreviewGraphSearch,
  createBrowserPreviewGraphStatus,
} from "../../browserPreviewData";
import type {
  GraphContextPack,
  GraphManifestSnapshot,
  GraphSearchSnapshot,
  GraphStatusSnapshot,
} from "../../types";
import { isBrowserPreviewRuntime } from "./useProjectFiles";

export type ProjectGraphState = {
  status: GraphStatusSnapshot | null;
  manifest: GraphManifestSnapshot | null;
  latestSearch: GraphSearchSnapshot | null;
  latestContextPack: GraphContextPack | null;
  error: string | null;
  source: "idle" | "loading" | "tauri" | "preview" | "unavailable";
};

const initialProjectGraphState: ProjectGraphState = {
  status: null,
  manifest: null,
  latestSearch: null,
  latestContextPack: null,
  error: null,
  source: "idle",
};

function readableProjectGraphError(error: unknown) {
  if (isBrowserPreviewRuntime()) {
    return "当前为浏览器预览，正在显示 Mock 代码地图；真实索引请在桌面客户端查看。";
  }
  return error instanceof Error ? error.message : String(error);
}

export function useProjectGraph(projectRoot: string | null) {
  const [projectGraphState, setProjectGraphState] = useState<ProjectGraphState>(initialProjectGraphState);

  const loadProjectGraph = useCallback(
    async (root = projectRoot) => {
      if (!root) {
        setProjectGraphState(initialProjectGraphState);
        return;
      }
      setProjectGraphState((current) => ({ ...current, error: null, source: "loading" }));
      try {
        const [status, manifest] = await Promise.all([
          invoke<GraphStatusSnapshot>("load_project_graph_status", { projectRoot: root }),
          invoke<GraphManifestSnapshot>("load_project_graph_manifest", { projectRoot: root }),
        ]);
        setProjectGraphState((current) => ({
          ...current,
          status,
          manifest,
          error: null,
          source: "tauri",
        }));
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          setProjectGraphState((current) => ({
            ...current,
            status: createBrowserPreviewGraphStatus(root),
            manifest: createBrowserPreviewGraphManifest(root),
            error: null,
            source: "preview",
          }));
          return;
        }
        setProjectGraphState((current) => ({
          ...current,
          status: null,
          manifest: null,
          error: readableProjectGraphError(error),
          source: "unavailable",
        }));
      }
    },
    [projectRoot],
  );

  const prepareProjectGraph = useCallback(
    async (root = projectRoot) => {
      if (!root) {
        setProjectGraphState(initialProjectGraphState);
        return;
      }
      setProjectGraphState((current) => ({ ...current, error: null, source: "loading" }));
      try {
        const status = await invoke<GraphStatusSnapshot>("prepare_project_graph", { projectRoot: root });
        const manifest = await invoke<GraphManifestSnapshot>("load_project_graph_manifest", { projectRoot: root });
        setProjectGraphState((current) => ({
          ...current,
          status,
          manifest,
          error: null,
          source: "tauri",
        }));
        if (status.status === "indexing") {
          window.setTimeout(() => {
            void loadProjectGraph(root);
          }, 900);
        }
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          setProjectGraphState((current) => ({
            ...current,
            status: createBrowserPreviewGraphStatus(root),
            manifest: createBrowserPreviewGraphManifest(root),
            error: null,
            source: "preview",
          }));
          return;
        }
        setProjectGraphState((current) => ({
          ...current,
          status: null,
          manifest: null,
          error: readableProjectGraphError(error),
          source: "unavailable",
        }));
      }
    },
    [loadProjectGraph, projectRoot],
  );

  const searchProjectGraph = useCallback(
    async (query: string, limit = 20, root = projectRoot) => {
      if (!root) {
        return null;
      }
      try {
        const result = await invoke<GraphSearchSnapshot>("search_project_graph", {
          projectRoot: root,
          query,
          limit,
        });
        setProjectGraphState((current) => ({ ...current, latestSearch: result, error: null }));
        return result;
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const result = createBrowserPreviewGraphSearch(query);
          setProjectGraphState((current) => ({ ...current, latestSearch: result, error: null, source: "preview" }));
          return result;
        }
        setProjectGraphState((current) => ({ ...current, error: readableProjectGraphError(error) }));
        return null;
      }
    },
    [projectRoot],
  );

  const buildGraphContextPack = useCallback(
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
        const pack = await invoke<GraphContextPack>("build_graph_context_pack", {
          projectRoot: root,
          targetType,
          targetId,
          title,
          objective,
          acceptanceCriteria,
        });
        setProjectGraphState((current) => ({ ...current, latestContextPack: pack, error: null }));
        return pack;
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const pack = createBrowserPreviewGraphContextPack(root);
          setProjectGraphState((current) => ({
            ...current,
            latestContextPack: pack,
            error: null,
            source: "preview",
          }));
          return pack;
        }
        setProjectGraphState((current) => ({ ...current, error: readableProjectGraphError(error) }));
        return null;
      }
    },
    [projectRoot],
  );

  useEffect(() => {
    void prepareProjectGraph(projectRoot);
  }, [prepareProjectGraph, projectRoot]);

  return {
    buildGraphContextPack,
    loadProjectGraph,
    prepareProjectGraph,
    projectGraphState,
    searchProjectGraph,
  };
}
