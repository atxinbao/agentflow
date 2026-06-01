import { invoke } from "@tauri-apps/api/core";
import type { Dispatch, SetStateAction } from "react";
import { useCallback } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewProjectDirectoryPage,
} from "../../../browserPreviewData";
import type { ProjectDirectoryPage, ProjectFileViewMode } from "../../../types";
import type { ProjectFilesState } from "../model/projectFileTypes";
import { isBrowserPreviewRuntime, readableProjectFilesError } from "./projectFileRuntime";

export function useProjectDirectoryPageLoader({
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
    async (directoryPath: string, cursor?: string | null) => {
      const projectRoot = getProjectRoot() ?? selectedProjectRoot;
      const viewMode = getViewMode();
      if (isBrowserPreviewRuntime()) {
        const page = createBrowserPreviewProjectDirectoryPage(
          directoryPath,
          projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT,
          viewMode,
          cursor,
        );
        setProjectFilesState((current) => ({
          ...current,
          directoryPages: {
            ...current.directoryPages,
            [directoryPath]: mergeDirectoryPages(current.directoryPages[directoryPath], page),
          },
          error: null,
          source: "preview",
        }));
        return page;
      }
      try {
        const page = await invoke<ProjectDirectoryPage>("load_project_directory_page", {
          projectRoot,
          directoryPath,
          cursor,
          viewMode,
        });
        setProjectFilesState((current) => ({
          ...current,
          directoryPages: {
            ...current.directoryPages,
            [directoryPath]: mergeDirectoryPages(current.directoryPages[directoryPath], page),
          },
          error: null,
          source: current.source === "loading" ? "tauri" : current.source,
        }));
        return page;
      } catch (error) {
        setProjectFilesState((current) => ({
          ...current,
          error: readableProjectFilesError(error),
        }));
        return null;
      }
    },
    [getProjectRoot, getViewMode, selectedProjectRoot, setProjectFilesState],
  );
}

function mergeDirectoryPages(previous: ProjectDirectoryPage | undefined, next: ProjectDirectoryPage): ProjectDirectoryPage {
  if (!previous) {
    return next;
  }
  const seen = new Set(previous.entries.map((entry) => entry.relativePath));
  const entries = [...previous.entries];
  next.entries.forEach((entry) => {
    if (!seen.has(entry.relativePath)) {
      seen.add(entry.relativePath);
      entries.push(entry);
    }
  });
  return {
    ...next,
    entries,
  };
}
