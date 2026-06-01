import { invoke } from "@tauri-apps/api/core";
import { useCallback } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewProjectFileTextRange,
} from "../../../browserPreviewData";
import type { ProjectFileTextRange } from "../../../types";
import { isBrowserPreviewRuntime, readableProjectFilesError } from "./projectFileRuntime";

export function useProjectFileTextRange({
  getProjectRoot,
  selectedProjectRoot,
}: {
  getProjectRoot: () => string | null;
  selectedProjectRoot: string | null;
}) {
  return useCallback(
    async (relativePath: string, startLine: number, lineCount: number): Promise<ProjectFileTextRange> => {
      const projectRoot = getProjectRoot() ?? selectedProjectRoot;
      if (isBrowserPreviewRuntime()) {
        return createBrowserPreviewProjectFileTextRange(
          relativePath,
          startLine,
          lineCount,
          projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT,
        );
      }
      try {
        return await invoke<ProjectFileTextRange>("load_project_file_text_range", {
          projectRoot,
          relativePath,
          startLine,
          lineCount,
        });
      } catch (error) {
        throw new Error(readableProjectFilesError(error));
      }
    },
    [getProjectRoot, selectedProjectRoot],
  );
}
