import type { ProjectFileViewMode } from "../model/projectFileTypes";
import { normalizeProjectRelativePath } from "../model/projectFileUtils";

const PROJECT_FILE_READER_STATE_KEY = "agentflow.projectFileReaderState.v1";

export const RECENT_FILE_LIMIT = 20;
export const DEFAULT_PROJECT_FILE_VIEW_MODE: ProjectFileViewMode = "all";

export type PersistedProjectFileReaderProjectState = {
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

export function readPersistedProjectFileReaderState(
  projectRoot: string | null,
): PersistedProjectFileReaderProjectState {
  if (typeof window === "undefined") {
    return {};
  }
  try {
    const raw = window.localStorage.getItem(PROJECT_FILE_READER_STATE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as PersistedProjectFileReaderStore &
      PersistedProjectFileReaderProjectState;
    const projectState = projectRoot ? parsed.projects?.[projectRoot] : null;
    if (projectState) {
      return normalizePersistedProjectFileReaderState(projectState);
    }

    return normalizePersistedProjectFileReaderState(parsed);
  } catch {
    return {};
  }
}

export function persistProjectFileReaderState(
  projectRoot: string,
  state: PersistedProjectFileReaderProjectState,
) {
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

export function readExpandedProjectFilePaths(projectRoot?: string | null) {
  if (typeof window === "undefined") {
    return new Set<string>();
  }

  try {
    const raw = window.localStorage.getItem(PROJECT_FILE_READER_STATE_KEY);
    const parsed = raw ? (JSON.parse(raw) as PersistedProjectFileReaderStore & { expandedPaths?: string[] }) : {};
    const expandedPaths = projectRoot ? parsed.projects?.[projectRoot]?.expandedPaths : parsed.expandedPaths;
    return new Set((expandedPaths ?? []).map(normalizeProjectRelativePath));
  } catch {
    return new Set<string>();
  }
}

export function persistExpandedProjectFilePaths(projectRoot: string, paths: ReadonlySet<string>) {
  persistProjectFileReaderState(projectRoot, {
    expandedPaths: [...paths],
  });
}

function isProjectFileViewMode(value: unknown): value is ProjectFileViewMode {
  return value === "source" || value === "all" || value === "recent";
}

function normalizePersistedProjectFileReaderState(
  state: PersistedProjectFileReaderProjectState,
): PersistedProjectFileReaderProjectState {
  return {
    projectRoot: state.projectRoot,
    selectedPath: state.selectedPath ?? null,
    viewMode: isProjectFileViewMode(state.viewMode) ? state.viewMode : DEFAULT_PROJECT_FILE_VIEW_MODE,
    recentPaths: Array.isArray(state.recentPaths) ? state.recentPaths.slice(0, RECENT_FILE_LIMIT) : [],
    expandedPaths: Array.isArray(state.expandedPaths) ? state.expandedPaths : [],
    lastOpenedAt: state.lastOpenedAt,
  };
}
