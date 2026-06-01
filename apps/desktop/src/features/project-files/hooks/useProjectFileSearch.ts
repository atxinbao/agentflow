import { invoke } from "@tauri-apps/api/core";
import type { Dispatch, SetStateAction } from "react";
import { useCallback } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewProjectFileSearchSnapshot,
} from "../../../browserPreviewData";
import type { ProjectFileSearchSnapshot, ProjectFileViewMode } from "../../../types";
import type { ProjectFilesState } from "../model/projectFileTypes";
import { isBrowserPreviewRuntime, readableProjectFilesError } from "./projectFileRuntime";

export function useProjectFileSearch({
  getProjectRoot,
  getViewMode,
  selectedProjectRoot,
  setProjectFilesState,
}: {
  getProjectRoot: () => string | null;
  getViewMode: () => ProjectFileViewMode;
  selectedProjectRoot: string | null;
  setProjectFilesState: Dispatch<SetStateAction<ProjectFilesState>>;
}) {
  return useCallback(
    async (query: string) => {
      const normalizedQuery = query.trim();
      const projectRoot = getProjectRoot() ?? selectedProjectRoot;
      const viewMode = getViewMode();
      setProjectFilesState((current) => ({
        ...current,
        searchQuery: query,
        searchLoading: Boolean(normalizedQuery),
        searchSnapshot: normalizedQuery ? current.searchSnapshot : null,
      }));
      if (!normalizedQuery) {
        return null;
      }
      if (isBrowserPreviewRuntime()) {
        const searchSnapshot = createBrowserPreviewProjectFileSearchSnapshot(
          normalizedQuery,
          projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT,
          viewMode,
        );
        setProjectFilesState((current) => ({
          ...current,
          searchSnapshot,
          searchLoading: false,
          error: null,
          source: "preview",
        }));
        return searchSnapshot;
      }
      try {
        const searchSnapshot = await invoke<ProjectFileSearchSnapshot>("search_project_files", {
          projectRoot,
          query: normalizedQuery,
          viewMode,
        });
        setProjectFilesState((current) => ({
          ...current,
          searchSnapshot,
          searchLoading: false,
          error: null,
        }));
        return searchSnapshot;
      } catch (error) {
        setProjectFilesState((current) => ({
          ...current,
          searchLoading: false,
          error: readableProjectFilesError(error),
        }));
        return null;
      }
    },
    [getProjectRoot, getViewMode, selectedProjectRoot, setProjectFilesState],
  );
}
