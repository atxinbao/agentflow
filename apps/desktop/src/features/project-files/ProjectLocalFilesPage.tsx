import { useEffect, useMemo, useState } from "react";
import { ProjectFileBrowser } from "./ProjectFileBrowser";
import { ProjectFileReader } from "./ProjectFileReader";
import "./ProjectFiles.css";
import type { ProjectFileBrowserRow, ProjectFileViewMode, ProjectFilesState } from "./projectFileTypes";
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
  onSearchFiles,
  onSelectFile,
  recommendedFilePaths,
}: {
  fileState: ProjectFilesState;
  onChangeViewMode: (viewMode: ProjectFileViewMode) => void;
  onLoadDirectoryPage: (directoryPath: string, cursor?: string | null) => Promise<unknown>;
  onSearchFiles: (query: string) => Promise<unknown>;
  onSelectFile: (relativePath: string) => void;
  recommendedFilePaths: string[];
}) {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => readExpandedPaths());
  const [searchDraft, setSearchDraft] = useState(fileState.searchQuery);
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
    if (!filesSnapshot || recommendedFilePaths.length === 0) {
      return [];
    }

    return recommendedFilePaths.slice(0, 6).map((recommendedPath): ProjectFileBrowserRow => {
      const normalizedPath = normalizeProjectRelativePath(recommendedPath);
      const entry = findProjectFileEntry(filesSnapshot.entries, normalizedPath);
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
        };
      }

      const name = normalizedPath.split("/").pop() ?? normalizedPath;
      return {
        name,
        relativePath: normalizedPath,
        kind: "file",
        createdAt: null,
        modifiedAt: null,
        sizeBytes: null,
        childCount: null,
        isSymlink: false,
        extension: getProjectFileExtensionFromName(name),
        children: [],
        depth: 0,
      };
    });
  }, [filesSnapshot, recommendedFilePaths]);

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
        recommendedFilePaths={recommendedFilePaths}
        recommendedRows={recommendedRows}
        viewMode={fileState.viewMode}
      />
    </section>
  );
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
