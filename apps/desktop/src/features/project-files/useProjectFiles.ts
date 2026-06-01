import { invoke } from "@tauri-apps/api/core";
import { useCallback, useRef, useState } from "react";
import {
  BROWSER_PREVIEW_PROJECT_ROOT,
  createBrowserPreviewProjectDirectoryPage,
  createBrowserPreviewProjectFileContent,
  createBrowserPreviewProjectFileSearchSnapshot,
  createBrowserPreviewProjectFilesSnapshot,
} from "../../browserPreviewData";
import type {
  ProjectDirectoryPage,
  ProjectFileContent,
  ProjectFileSearchSnapshot,
  ProjectFileViewMode,
  ProjectFilesSnapshot,
} from "../../types";
import type { ProjectFilesState } from "./projectFileTypes";
import { findProjectFileEntry, normalizeProjectRelativePath } from "./projectFileUtils";

const PROJECT_FILE_READER_STATE_KEY = "agentflow.projectFileReaderState.v1";
const RECENT_FILE_LIMIT = 20;
const DEFAULT_PROJECT_FILE_VIEW_MODE: ProjectFileViewMode = "source";

type PersistedProjectFileReaderProjectState = {
  projectRoot?: string;
  selectedPath?: string | null;
  viewMode?: ProjectFileViewMode;
  expandedPaths?: string[];
  recentPaths?: string[];
  lastOpenedAt?: string;
};

type PersistedProjectFileReaderStore = {
  version?: number;
  projects?: Record<string, PersistedProjectFileReaderProjectState>;
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
  const [projectFilesState, setProjectFilesState] = useState<ProjectFilesState>(() => {
    const persisted = readPersistedProjectFileReaderState(selectedProjectRoot);
    return {
      snapshot: null,
      content: null,
      selectedPath: persisted.selectedPath ?? null,
      error: null,
      source: "loading",
      viewMode: persisted.viewMode ?? DEFAULT_PROJECT_FILE_VIEW_MODE,
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
        if (isBrowserPreviewRuntime()) {
          const filesSnapshot = createBrowserPreviewProjectFilesSnapshot(projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT, viewMode);
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

  const loadProjectDirectoryPage = useCallback(
    async (directoryPath: string, cursor?: string | null) => {
      const projectRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot;
      const viewMode = projectFilesState.viewMode;
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
        if (isBrowserPreviewRuntime()) {
          const page = createBrowserPreviewProjectDirectoryPage(directoryPath, projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT, viewMode, cursor);
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
        setProjectFilesState((current) => ({
          ...current,
          error: readableProjectFilesError(error),
        }));
        return null;
      }
    },
    [projectFilesState.snapshot?.projectRoot, projectFilesState.viewMode, selectedProjectRoot],
  );

  const searchProjectFiles = useCallback(
    async (query: string) => {
      const normalizedQuery = query.trim();
      const projectRoot = projectFilesState.snapshot?.projectRoot ?? selectedProjectRoot;
      setProjectFilesState((current) => ({
        ...current,
        searchQuery: query,
        searchLoading: Boolean(normalizedQuery),
        searchSnapshot: normalizedQuery ? current.searchSnapshot : null,
      }));
      if (!normalizedQuery) {
        return null;
      }
      try {
        const searchSnapshot = await invoke<ProjectFileSearchSnapshot>("search_project_files", {
          projectRoot,
          query: normalizedQuery,
          viewMode: projectFilesState.viewMode,
        });
        setProjectFilesState((current) => ({
          ...current,
          searchSnapshot,
          searchLoading: false,
          error: null,
        }));
        return searchSnapshot;
      } catch (error) {
        if (isBrowserPreviewRuntime()) {
          const searchSnapshot = createBrowserPreviewProjectFileSearchSnapshot(
            normalizedQuery,
            projectRoot ?? BROWSER_PREVIEW_PROJECT_ROOT,
            projectFilesState.viewMode,
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
        setProjectFilesState((current) => ({
          ...current,
          searchLoading: false,
          error: readableProjectFilesError(error),
        }));
        return null;
      }
    },
    [projectFilesState.snapshot?.projectRoot, projectFilesState.viewMode, selectedProjectRoot],
  );

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
    loadProjectFiles,
    projectFilesState,
    reportProjectFilesError,
    searchProjectFiles,
    selectProjectFile,
    setProjectFileViewMode,
  };
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

function readPersistedProjectFileReaderState(projectRoot: string | null): PersistedProjectFileReaderProjectState {
  if (typeof window === "undefined") {
    return {};
  }
  try {
    const raw = window.localStorage.getItem(PROJECT_FILE_READER_STATE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as PersistedProjectFileReaderStore & PersistedProjectFileReaderProjectState;
    const projectState = projectRoot ? parsed.projects?.[projectRoot] : null;
    if (projectState) {
      return normalizePersistedProjectFileReaderState(projectState);
    }

    return normalizePersistedProjectFileReaderState(parsed);
  } catch {
    return {};
  }
}

function persistProjectFileReaderState(projectRoot: string, state: PersistedProjectFileReaderProjectState) {
  if (typeof window === "undefined") {
    return;
  }
  try {
    const raw = window.localStorage.getItem(PROJECT_FILE_READER_STATE_KEY);
    const parsed = raw ? (JSON.parse(raw) as PersistedProjectFileReaderStore) : {};
    const projects = parsed.projects ?? {};
    const previous = projects[projectRoot] ?? {};
    projects[projectRoot] = {
      ...previous,
      ...state,
      projectRoot,
      lastOpenedAt: new Date().toISOString(),
    };
    window.localStorage.setItem(
      PROJECT_FILE_READER_STATE_KEY,
      JSON.stringify({
        version: 1,
        projects,
      }),
    );
  } catch {
    // 本地 UI 状态持久化失败不影响只读文件浏览。
  }
}

function isProjectFileViewMode(value: unknown): value is ProjectFileViewMode {
  return value === "source" || value === "all" || value === "recent";
}

function normalizePersistedProjectFileReaderState(state: PersistedProjectFileReaderProjectState): PersistedProjectFileReaderProjectState {
  return {
    projectRoot: state.projectRoot,
    selectedPath: state.selectedPath ?? null,
    viewMode: isProjectFileViewMode(state.viewMode) ? state.viewMode : DEFAULT_PROJECT_FILE_VIEW_MODE,
    recentPaths: Array.isArray(state.recentPaths) ? state.recentPaths.slice(0, RECENT_FILE_LIMIT) : [],
    expandedPaths: Array.isArray(state.expandedPaths) ? state.expandedPaths : [],
    lastOpenedAt: state.lastOpenedAt,
  };
}
