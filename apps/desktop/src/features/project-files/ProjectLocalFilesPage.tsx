import { useEffect, useMemo, useState } from "react";
import { ProjectFileBrowser } from "./browser/ProjectFileBrowser";
import { ProjectFileReader } from "./reader/ProjectFileReader";
import "./ProjectFiles.css";
import type {
  ProjectFileBrowserRow,
  ProjectFileTextRange,
  ProjectFileViewMode,
  ProjectFilesState,
} from "./model/projectFileTypes";
import { buildProjectFileBrowserRows } from "./browser/projectFileBrowserRows";
import {
  findProjectFileEntry,
  getProjectFileExtensionFromName,
  normalizeProjectRelativePath,
  projectFileChildToEntry,
} from "./model/projectFileUtils";
import {
  persistExpandedProjectFilePaths,
  readExpandedProjectFilePaths,
} from "./hooks/projectFileReaderState";

export function ProjectLocalFilesPage({
  fileState,
  onChangeViewMode,
  onLoadDirectoryPage,
  onLoadTextRange,
  onSearchFiles,
  onSelectFile,
}: {
  fileState: ProjectFilesState;
  onChangeViewMode: (viewMode: ProjectFileViewMode) => void;
  onLoadDirectoryPage: (directoryPath: string, cursor?: string | null) => Promise<unknown>;
  onLoadTextRange?: (relativePath: string, startLine: number, lineCount: number) => Promise<ProjectFileTextRange>;
  onSearchFiles: (query: string) => Promise<unknown>;
  onSelectFile: (relativePath: string) => void;
}) {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => readExpandedProjectFilePaths());
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

  useEffect(() => {
    if (!filesSnapshot?.projectRoot) {
      return;
    }
    setExpandedPaths(readExpandedProjectFilePaths(filesSnapshot.projectRoot));
  }, [filesSnapshot?.projectRoot]);

  useEffect(() => {
    if (!filesSnapshot?.projectRoot) {
      return;
    }
    persistExpandedProjectFilePaths(filesSnapshot.projectRoot, expandedPaths);
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
