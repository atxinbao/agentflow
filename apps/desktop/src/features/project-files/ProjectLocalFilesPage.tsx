import { useEffect, useMemo, useState } from "react";
import { ProjectFileBrowser } from "./ProjectFileBrowser";
import { ProjectFileReader } from "./ProjectFileReader";
import "./ProjectFiles.css";
import type { ProjectGraphState } from "./useProjectGraph";
import type {
  ProjectFileBrowserRow,
  ProjectFileEntry,
  ProjectFileTextRange,
  ProjectFileViewMode,
  ProjectFilesState,
  ProjectRecommendedFile,
} from "./projectFileTypes";
import {
  buildProjectFileBrowserRows,
  findProjectFileEntry,
  getProjectFileExtensionFromName,
  normalizeProjectRelativePath,
  projectFileChildToEntry,
} from "./projectFileUtils";

const PROJECT_FILE_READER_STATE_KEY = "agentflow.projectFileReaderState.v1";

export function ProjectLocalFilesPage({
  fileState,
  onChangeViewMode,
  onLoadDirectoryPage,
  onLoadTextRange,
  onSearchFiles,
  onSelectFile,
  graphState,
}: {
  fileState: ProjectFilesState;
  graphState: ProjectGraphState;
  onChangeViewMode: (viewMode: ProjectFileViewMode) => void;
  onLoadDirectoryPage: (directoryPath: string, cursor?: string | null) => Promise<unknown>;
  onLoadTextRange?: (relativePath: string, startLine: number, lineCount: number) => Promise<ProjectFileTextRange>;
  onSearchFiles: (query: string) => Promise<unknown>;
  onSelectFile: (relativePath: string) => void;
}) {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => readExpandedPaths());
  const [searchDraft, setSearchDraft] = useState(fileState.searchQuery);
  const [recommendedFileWarning, setRecommendedFileWarning] = useState<string | null>(null);
  const filesSnapshot = fileState.snapshot;
  const selectedPath = filesSnapshot
    ? fileState.selectedPath ?? filesSnapshot.selectedPath ?? filesSnapshot.entries.at(0)?.relativePath ?? null
    : fileState.selectedPath;
  const selectedEntry = filesSnapshot
    ? selectedPath
      ? findProjectFileEntry(filesSnapshot.entries, selectedPath)
      : filesSnapshot.entries.at(0) ?? null
    : null;
  const selectedContent = fileState.content?.relativePath === selectedPath ? fileState.content : null;
  const content = selectedContent;
  const fileRows = useMemo(() => {
    if (!filesSnapshot) {
      return [];
    }
    if (fileState.searchSnapshot?.query) {
      return fileState.searchSnapshot.results.map((result): ProjectFileBrowserRow => ({
        name: result.name,
        relativePath: result.relativePath,
        kind: result.kind,
        createdAt: null,
        modifiedAt: result.modifiedAt ?? null,
        sizeBytes: result.sizeBytes ?? null,
        childCount: null,
        isSymlink: false,
        extension: result.extension ?? getProjectFileExtensionFromName(result.name),
        children: [],
        depth: 0,
      }));
    }
    if (fileState.viewMode === "recent") {
      const recentRows: ProjectFileBrowserRow[] = [];
      fileState.recentPaths.forEach((recentPath) => {
        const entry = findProjectFileEntry(filesSnapshot.entries, recentPath);
        if (!entry) {
          return;
        }
        recentRows.push({
          name: entry.name,
          relativePath: entry.relativePath,
          kind: entry.kind,
          createdAt: entry.createdAt,
          modifiedAt: entry.modifiedAt,
          sizeBytes: entry.sizeBytes,
          childCount: entry.childCount,
          isSymlink: entry.isSymlink ?? false,
          extension: entry.extension,
          children: entry.children,
          depth: 0,
        });
      });
      return recentRows;
    }
    return buildProjectFileBrowserRows(filesSnapshot.entries, expandedPaths, fileState.directoryPages, content);
  }, [content, expandedPaths, fileState.directoryPages, fileState.recentPaths, fileState.searchSnapshot, fileState.viewMode, filesSnapshot]);

  const recommendedRows = useMemo(() => {
    return buildRecommendedFileRows(graphState, filesSnapshot?.entries ?? []);
  }, [filesSnapshot?.entries, graphState]);

  useEffect(() => {
    if (!filesSnapshot?.projectRoot) {
      return;
    }
    setExpandedPaths(readExpandedPaths(filesSnapshot.projectRoot));
  }, [filesSnapshot?.projectRoot]);

  useEffect(() => {
    if (!filesSnapshot?.projectRoot) {
      return;
    }
    persistExpandedPaths(filesSnapshot.projectRoot, expandedPaths);
  }, [expandedPaths, filesSnapshot?.projectRoot]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      void onSearchFiles(searchDraft);
    }, 180);
    return () => window.clearTimeout(timeout);
  }, [onSearchFiles, searchDraft]);

  async function handleProjectFileRowSelect(row: ProjectFileBrowserRow) {
    if (row.missing) {
      setRecommendedFileWarning(`${row.relativePath} 已不在当前项目文件树中。`);
      return;
    }
    setRecommendedFileWarning(null);

    if (row.hasMoreChildren) {
      const directoryPath = row.relativePath.replace(/::__load_more__$/, "");
      await onLoadDirectoryPage(directoryPath, row.nextCursor);
      return;
    }

    if (row.kind === "directory") {
      setExpandedPaths((current) => {
        const next = new Set(current);
        if (next.has(row.relativePath)) {
          next.delete(row.relativePath);
        } else {
          next.add(row.relativePath);
          if (!fileState.directoryPages[row.relativePath]) {
            void onLoadDirectoryPage(row.relativePath);
          }
        }
        return next;
      });
    }
    onSelectFile(row.relativePath);
  }

  return (
    <section className="project-file-page" aria-label="项目本地文件阅读器">
      <article className="project-file-reader" aria-label="文件内容阅读器">
        <ProjectFileReader
          content={content}
          entry={selectedEntry ?? (content?.kind === "directory" ? projectFileChildToEntry(contentToChild(content)) : null)}
          error={fileState.error}
          loading={fileState.loading}
          loadingPath={fileState.loadingPath}
          onLoadTextRange={onLoadTextRange}
        />
      </article>

      <ProjectFileBrowser
        expandedPaths={expandedPaths}
        loading={fileState.loading}
        onChangeViewMode={onChangeViewMode}
        onSearchChange={setSearchDraft}
        onSelectRow={(row) => void handleProjectFileRowSelect(row)}
        rows={fileRows}
        searchLoading={fileState.searchLoading}
        searchQuery={searchDraft}
        selectedPath={selectedPath}
        recommendedFileWarning={recommendedFileWarning}
        recommendedRows={recommendedRows}
        viewMode={fileState.viewMode}
      />
    </section>
  );
}

function buildRecommendedFileRows(graphState: ProjectGraphState, entries: ProjectFileEntry[]): ProjectFileBrowserRow[] {
  const recommendedFiles = new Map<string, ProjectRecommendedFile>();

  graphState.latestContextPack?.recommendedFiles.forEach((file) => {
    const path = normalizeProjectRelativePath(file.path);
    recommendedFiles.set(path, {
      path,
      name: path.split("/").pop() ?? path,
      source: "context-pack-file",
      reason: file.reason,
      status: "unloaded",
    });
  });

  graphState.latestContextPack?.recommendedTests.forEach((file) => {
    const path = normalizeProjectRelativePath(file.path);
    if (!recommendedFiles.has(path)) {
      recommendedFiles.set(path, {
        path,
        name: path.split("/").pop() ?? path,
        source: "context-pack-test",
        reason: file.reason,
        status: "unloaded",
      });
    }
  });

  graphState.manifest?.importantFiles.forEach((filePath) => {
    const path = normalizeProjectRelativePath(filePath);
    if (!recommendedFiles.has(path)) {
      recommendedFiles.set(path, {
        path,
        name: path.split("/").pop() ?? path,
        source: "manifest-important",
        reason: "代码地图清单标记为重要文件。",
        status: "unloaded",
      });
    }
  });

  return [...recommendedFiles.values()].slice(0, 8).map((file) => recommendedFileToRow(file, entries));
}

function recommendedFileToRow(file: ProjectRecommendedFile, entries: ProjectFileEntry[]): ProjectFileBrowserRow {
  const entry = findProjectFileEntry(entries, file.path);
  if (entry) {
    return {
      name: entry.name,
      relativePath: entry.relativePath,
      kind: entry.kind,
      createdAt: entry.createdAt,
      modifiedAt: entry.modifiedAt,
      sizeBytes: entry.sizeBytes,
      childCount: entry.childCount,
      isSymlink: entry.isSymlink ?? false,
      extension: entry.extension,
      children: entry.children,
      depth: 0,
      recommendation: { ...file, status: "available" },
    };
  }

  const name = file.name || file.path.split("/").pop() || file.path;
  return {
    name,
    relativePath: file.path,
    kind: "file",
    createdAt: null,
    modifiedAt: null,
    sizeBytes: null,
    childCount: null,
    isSymlink: false,
    extension: getProjectFileExtensionFromName(name),
    children: [],
    depth: 0,
    missing: true,
    recommendation: { ...file, status: "missing" },
  };
}

function contentToChild(content: NonNullable<ProjectFilesState["content"]>) {
  return {
    name: content.name,
    relativePath: normalizeProjectRelativePath(content.relativePath),
    kind: content.kind,
    createdAt: content.createdAt,
    modifiedAt: content.modifiedAt,
    sizeBytes: content.sizeBytes,
    extension: content.extension,
    childCount: content.directoryChildren.length,
    isSymlink: false,
  };
}

function readExpandedPaths(projectRoot?: string | null) {
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

function persistExpandedPaths(projectRoot: string, paths: ReadonlySet<string>) {
  if (typeof window === "undefined") {
    return;
  }
  try {
    const raw = window.localStorage.getItem(PROJECT_FILE_READER_STATE_KEY);
    const parsed = raw ? (JSON.parse(raw) as PersistedProjectFileReaderStore) : {};
    const projects = parsed.projects ?? {};
    projects[projectRoot] = {
      ...(projects[projectRoot] ?? {}),
      projectRoot,
      expandedPaths: [...paths],
      lastOpenedAt: new Date().toISOString(),
    };
    window.localStorage.setItem(PROJECT_FILE_READER_STATE_KEY, JSON.stringify({ version: 1, projects }));
  } catch {
    // 本地持久化失败不影响只读文件浏览。
  }
}

type PersistedProjectFileReaderStore = {
  projects?: Record<string, { expandedPaths?: string[]; projectRoot?: string; lastOpenedAt?: string }>;
};
