import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewProjectFileContent,
  createBrowserPreviewProjectFileTextRange,
  createBrowserPreviewProjectFilesSnapshot,
} from "../../browserPreviewData";
import type { ProjectFileContent, ProjectFileTextRange, ProjectFilesSnapshot } from "../../types";
import type { ProjectFilesState } from "./projectFileTypes";
import { findProjectFileEntry } from "./projectFileUtils";

const initialProjectFilesState: ProjectFilesState = {
  snapshot: null,
  content: null,
  selectedPath: null,
  error: null,
  source: "loading",
};

export function isBrowserPreviewRuntime() {
  if (typeof window === "undefined") {
    return false;
  }

  return !("__TAURI_INTERNALS__" in window);
}

function readableProjectFilesError(error: unknown) {
  if (isBrowserPreviewRuntime()) {
    return "当前为浏览器预览，正在显示 Mock 项目文件；真实内容请在桌面客户端查看。";
  }
  return error instanceof Error ? error.message : String(error);
}

export function useProjectFiles(selectedProjectRoot: string | null) {
  const [projectFilesState, setProjectFilesState] = useState<ProjectFilesState>(initialProjectFilesState);

  const selectProjectFile = useCallback(
    async (relativePath: string) => {
      const projectRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot;
      setProjectFilesState((current) => ({ ...current, selectedPath: relativePath, error: null }));
      try {
        const content = await invoke<ProjectFileContent>("load_project_file_content", { projectRoot, relativePath });
        setProjectFilesState((current) => ({
          ...current,
          content,
          selectedPath: relativePath,
          error: null,
          source: "tauri",
        }));
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const previewProjectRoot = projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT;
          const content = createBrowserPreviewProjectFileContent(relativePath, previewProjectRoot);
          setProjectFilesState((current) => ({
            ...current,
            content,
            selectedPath: content.relativePath,
            error: null,
            source: "preview",
          }));
          return;
        }
        const errorMessage = readableProjectFilesError(error);
        setProjectFilesState((current) => ({
          ...current,
          content: null,
          selectedPath: relativePath,
          error: errorMessage,
          source: "unavailable",
        }));
      }
    },
    [projectFilesState.snapshot?.projectRoot, selectedProjectRoot],
  );

  const loadProjectFiles = useCallback(
    async (projectRoot = selectedProjectRoot, selectedPathHint = projectFilesState.selectedPath) => {
      setProjectFilesState((current) => ({ ...current, error: null, source: "loading" }));
      try {
        const filesSnapshot = await invoke<ProjectFilesSnapshot>("load_project_files_snapshot", { projectRoot });
        const selectedPath =
          selectedPathHint && findProjectFileEntry(filesSnapshot.entries, selectedPathHint)
            ? selectedPathHint
            : filesSnapshot.selectedPath ?? filesSnapshot.entries.at(0)?.relativePath ?? null;
        let content: ProjectFileContent | null = null;
        if (selectedPath) {
          content = await invoke<ProjectFileContent>("load_project_file_content", {
            projectRoot: filesSnapshot.projectRoot,
            relativePath: selectedPath,
          });
        }
        setProjectFilesState({
          snapshot: filesSnapshot,
          content,
          selectedPath,
          error: null,
          source: "tauri",
        });
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const filesSnapshot = createBrowserPreviewProjectFilesSnapshot(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT);
          const selectedPath =
            selectedPathHint && findProjectFileEntry(filesSnapshot.entries, selectedPathHint)
              ? selectedPathHint
              : filesSnapshot.selectedPath ?? filesSnapshot.entries.at(0)?.relativePath ?? "README.md";
          setProjectFilesState({
            snapshot: filesSnapshot,
            content: createBrowserPreviewProjectFileContent(selectedPath, filesSnapshot.projectRoot),
            selectedPath,
            error: null,
            source: "preview",
          });
          return;
        }
        const errorMessage = readableProjectFilesError(error);
        setProjectFilesState({
          snapshot: null,
          content: null,
          selectedPath: null,
          error: errorMessage,
          source: "unavailable",
        });
      }
    },
    [projectFilesState.selectedPath, selectedProjectRoot],
  );

  const loadProjectFileTextRange = useCallback(
    async (relativePath: string, startLine: number, lineCount: number) => {
      const projectRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot;
      try {
        return await invoke<ProjectFileTextRange>("load_project_file_text_range", {
          projectRoot,
          relativePath,
          startLine,
          lineCount,
        });
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          return createBrowserPreviewProjectFileTextRange(
            relativePath,
            startLine,
            lineCount,
            projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT,
          );
        }
        throw new Error(readableProjectFilesError(error));
      }
    },
    [projectFilesState.snapshot?.projectRoot, selectedProjectRoot],
  );

  const reportProjectFilesError = useCallback((message: string) => {
    setProjectFilesState((current) => ({
      ...current,
      content: null,
      error: message,
      source: current.snapshot ? current.source : "unavailable",
    }));
  }, []);

  const clearProjectFilesError = useCallback(() => {
    setProjectFilesState((current) => ({ ...current, error: null }));
  }, []);

  return {
    projectFilesState,
    loadProjectFiles,
    selectProjectFile,
    loadProjectFileTextRange,
    reportProjectFilesError,
    clearProjectFilesError,
  };
}
