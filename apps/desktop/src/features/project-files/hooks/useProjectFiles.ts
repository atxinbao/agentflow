import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewProjectFileContent,
  createBrowserPreviewProjectFilesSnapshot,
} from "../../../browserPreviewData";
import type {
  ProjectFileContent,
  ProjectFileViewMode,
  ProjectFilesSnapshot,
} from "../../../types";
import type { ProjectFilesState } from "../model/projectFileTypes";
import { findProjectFileEntry, normalizeProjectRelativePath } from "../model/projectFileUtils";
import {
  DEFAULT_PROJECT_FILE_VIEW_MODE,
  RECENT_FILE_LIMIT,
  persistProjectFileReaderState,
  readPersistedProjectFileReaderState,
} from "./projectFileReaderState";
import { isBrowserPreviewRuntime, readableProjectFilesError } from "./projectFileRuntime";
import { useProjectDirectoryPageLoader } from "./useProjectDirectoryPages";
import { useProjectFileSearch } from "./useProjectFileSearch";
import { useProjectFileTextRange } from "./useProjectFileTextRange";

function emptyProjectFilesState(viewMode: ProjectFileViewMode): ProjectFilesState {
  return {
    snapshot: null,
    content: null,
    selectedPath: null,
    error: null,
    source: "idle",
    viewMode,
    loading: false,
    loadingPath: null,
    directoryPages: {},
    searchQuery: "",
    searchSnapshot: null,
    searchLoading: false,
    recentPaths: [],
  };
}

export function useProjectFiles(selectedProjectRoot: string | null) {
  const [projectFilesState, setProjectFilesState] = useState<ProjectFilesState>(() => {
    const persisted = readPersistedProjectFileReaderState(selectedProjectRoot);
    const initialViewMode = persisted.viewMode ?? DEFAULT_PROJECT_FILE_VIEW_MODE;
    if (!selectedProjectRoot) {
      return emptyProjectFilesState(initialViewMode);
    }
    if (isBrowserPreviewRuntime()) {
      const filesSnapshot = createBrowserPreviewProjectFilesSnapshot(selectedProjectRoot, initialViewMode);
      const selectedPath =
        persisted.selectedPath && findProjectFileEntry(filesSnapshot.entries, persisted.selectedPath)
          ? persisted.selectedPath
          : filesSnapshot.selectedPath ?? filesSnapshot.entries.at(0)?.relativePath ?? "README.md";
      return {
        snapshot: filesSnapshot,
        content: createBrowserPreviewProjectFileContent(selectedPath, filesSnapshot.projectRoot),
        selectedPath,
        error: null,
        source: "preview",
        viewMode: initialViewMode,
        loading: false,
        loadingPath: null,
        directoryPages: {},
        searchQuery: "",
        searchSnapshot: null,
        searchLoading: false,
        recentPaths: persisted.recentPaths ?? [],
      };
    }
    return {
      snapshot: null,
      content: null,
      selectedPath: persisted.selectedPath ?? null,
      error: null,
      source: "loading",
      viewMode: initialViewMode,
      loading: false,
      loadingPath: null,
      directoryPages: {},
      searchQuery: "",
      searchSnapshot: null,
      searchLoading: false,
      recentPaths: persisted.recentPaths ?? [],
    };
  });
  const requestCounter = useRef(0);

  useEffect(() => {
    if (selectedProjectRoot) {
      return;
    }
    requestCounter.current += 1;
    setProjectFilesState((current) => emptyProjectFilesState(current.viewMode));
  }, [selectedProjectRoot]);

  const currentProjectRoot = useCallback(
    () => projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot,
    [projectFilesState.snapshot?.projectRoot, selectedProjectRoot],
  );

  const currentViewMode = useCallback(
    () => projectFilesState.viewMode,
    [projectFilesState.viewMode],
  );

  const rememberProjectFile = useCallback((relativePath: string) => {
    setProjectFilesState((current) => {
      const projectRoot = current.snapshot?.projectRoot ?? selectedProjectRoot;
      const normalizedPath = normalizeProjectRelativePath(relativePath);
      const recentPaths = [normalizedPath, ...current.recentPaths.filter((path) => path !== normalizedPath)].slice(0, RECENT_FILE_LIMIT);
      if (projectRoot) {
        persistProjectFileReaderState(projectRoot, {
          selectedPath: normalizedPath,
          viewMode: current.viewMode,
          recentPaths,
        });
      }
      return {
        ...current,
        recentPaths,
      };
    });
  }, [selectedProjectRoot]);

  const selectProjectFile = useCallback(
    async (relativePath: string) => {
      const projectRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot;
      const requestId = requestCounter.current + 1;
      requestCounter.current = requestId;
      if (isBrowserPreviewRuntime()) {
        const previewProjectRoot = projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT;
        const content = createBrowserPreviewProjectFileContent(relativePath, previewProjectRoot);
        setProjectFilesState((current) => ({
          ...current,
          content,
          selectedPath: content.relativePath,
          error: null,
          source: "preview",
          loading: false,
          loadingPath: null,
        }));
        rememberProjectFile(content.relativePath);
        return;
      }
      setProjectFilesState((current) => ({
        ...current,
        selectedPath: relativePath,
        error: null,
        loading: true,
        loadingPath: relativePath,
      }));
      try {
        const content = await invoke<ProjectFileContent>("load_project_file_content", { projectRoot, relativePath });
        if (requestCounter.current !== requestId) {
          return;
        }
        setProjectFilesState((current) => ({
          ...current,
          content,
          selectedPath: relativePath,
          error: null,
          source: "tauri",
          loading: false,
          loadingPath: null,
        }));
        rememberProjectFile(relativePath);
      } catch (error) {
        if (requestCounter.current !== requestId) {
          return;
        }
        const errorMessage = readableProjectFilesError(error);
        setProjectFilesState((current) => ({
          ...current,
          content: null,
          selectedPath: relativePath,
          error: errorMessage,
          source: "unavailable",
          loading: false,
          loadingPath: null,
        }));
      }
    },
    [projectFilesState.snapshot?.projectRoot, rememberProjectFile, selectedProjectRoot],
  );

  const loadProjectFiles = useCallback(
    async (
      projectRoot = selectedProjectRoot,
      selectedPathHint?: string | null,
      viewModeHint?: ProjectFileViewMode,
    ) => {
      const persistedState = readPersistedProjectFileReaderState(projectRoot);
      const viewMode = viewModeHint ?? persistedState.viewMode ?? projectFilesState.viewMode;
      const selectedPathPreference = selectedPathHint ?? persistedState.selectedPath ?? projectFilesState.selectedPath;
      const requestId = requestCounter.current + 1;
      requestCounter.current = requestId;
      if (!projectRoot) {
        setProjectFilesState(emptyProjectFilesState(viewMode));
        return;
      }
      if (isBrowserPreviewRuntime()) {
        const filesSnapshot = createBrowserPreviewProjectFilesSnapshot(projectRoot, viewMode);
        const selectedPath =
          selectedPathPreference && findProjectFileEntry(filesSnapshot.entries, selectedPathPreference)
            ? selectedPathPreference
            : filesSnapshot.selectedPath ?? filesSnapshot.entries.at(0)?.relativePath ?? "README.md";
        setProjectFilesState((current) => ({
          ...current,
          snapshot: filesSnapshot,
          content: createBrowserPreviewProjectFileContent(selectedPath, filesSnapshot.projectRoot),
          selectedPath,
          error: null,
          source: "preview",
          viewMode,
          loading: false,
          loadingPath: null,
          directoryPages: {},
          searchSnapshot: null,
          recentPaths: persistedState.recentPaths ?? current.recentPaths,
        }));
        persistProjectFileReaderState(filesSnapshot.projectRoot, {
          selectedPath,
          viewMode,
          recentPaths: persistedState.recentPaths ?? projectFilesState.recentPaths,
        });
        return;
      }
      setProjectFilesState((current) => ({
        ...current,
        error: null,
        source: "loading",
        loading: true,
        loadingPath: selectedPathPreference ?? null,
        viewMode,
        recentPaths: persistedState.recentPaths ?? current.recentPaths,
        directoryPages: {},
        searchSnapshot: null,
      }));
      try {
        const filesSnapshot = await invoke<ProjectFilesSnapshot>("load_project_files_snapshot", { projectRoot, viewMode });
        const selectedPath =
          selectedPathPreference && findProjectFileEntry(filesSnapshot.entries, selectedPathPreference)
            ? selectedPathPreference
            : filesSnapshot.selectedPath ?? filesSnapshot.entries.at(0)?.relativePath ?? null;
        let content: ProjectFileContent | null = null;
        if (selectedPath) {
          content = await invoke<ProjectFileContent>("load_project_file_content", {
            projectRoot: filesSnapshot.projectRoot,
            relativePath: selectedPath,
          });
        }
        if (requestCounter.current !== requestId) {
          return;
        }
        setProjectFilesState((current) => ({
          ...current,
          snapshot: filesSnapshot,
          content,
          selectedPath,
          error: null,
          source: "tauri",
          viewMode,
          loading: false,
          loadingPath: null,
          directoryPages: {},
          searchSnapshot: null,
          recentPaths: persistedState.recentPaths ?? current.recentPaths,
        }));
        persistProjectFileReaderState(filesSnapshot.projectRoot, {
          selectedPath,
          viewMode,
          recentPaths: persistedState.recentPaths ?? projectFilesState.recentPaths,
        });
      } catch (error) {
        if (requestCounter.current !== requestId) {
          return;
        }
        const errorMessage = readableProjectFilesError(error);
        setProjectFilesState((current) => ({
          ...current,
          snapshot: null,
          content: null,
          selectedPath: null,
          error: errorMessage,
          source: "unavailable",
          loading: false,
          loadingPath: null,
        }));
      }
    },
    [projectFilesState.recentPaths, projectFilesState.selectedPath, projectFilesState.viewMode, selectedProjectRoot],
  );

  const setProjectFileViewMode = useCallback(
    (viewMode: ProjectFileViewMode) => {
      setProjectFilesState((current) => ({ ...current, viewMode }));
      const projectRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot;
      if (projectRoot) {
        persistProjectFileReaderState(projectRoot, {
          selectedPath: projectFilesState.selectedPath,
          viewMode,
          recentPaths: projectFilesState.recentPaths,
        });
      }
      void loadProjectFiles(projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot, projectFilesState.selectedPath, viewMode);
    },
    [loadProjectFiles, projectFilesState.recentPaths, projectFilesState.selectedPath, projectFilesState.snapshot?.projectRoot, selectedProjectRoot],
  );

  const loadProjectDirectoryPage = useProjectDirectoryPageLoader({
    getProjectRoot: currentProjectRoot,
    getViewMode: currentViewMode,
    selectedProjectRoot,
    setProjectFilesState,
  });

  const searchProjectFiles = useProjectFileSearch({
    getProjectRoot: currentProjectRoot,
    getViewMode: currentViewMode,
    selectedProjectRoot,
    setProjectFilesState,
  });

  const loadProjectFileTextRange = useProjectFileTextRange({
    getProjectRoot: currentProjectRoot,
    selectedProjectRoot,
  });

  const reportProjectFilesError = useCallback((message: string) => {
    setProjectFilesState((current) => ({
      ...current,
      content: null,
      error: message,
      source: current.snapshot ? current.source : "unavailable",
      loading: false,
      loadingPath: null,
    }));
  }, []);

  const clearProjectFilesError = useCallback(() => {
    setProjectFilesState((current) => ({ ...current, error: null }));
  }, []);

  return {
    clearProjectFilesError,
    loadProjectDirectoryPage,
    loadProjectFileTextRange,
    loadProjectFiles,
    projectFilesState,
    reportProjectFilesError,
    searchProjectFiles,
    selectProjectFile,
    setProjectFileViewMode,
  };
}
