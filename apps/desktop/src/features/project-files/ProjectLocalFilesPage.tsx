import { useEffect, useMemo, useState } from "react";
import { ProjectFileBrowser } from "./ProjectFileBrowser";
import { ProjectFileReader } from "./ProjectFileReader";
import "./ProjectFiles.css";
import type {
  ProjectFileBrowserRow,
  ProjectFileEntry,
  ProjectFileTextRange,
  ProjectFilesState,
  ProjectRecommendedFile,
} from "./projectFileTypes";
import type { ProjectGraphState } from "./useProjectGraph";
import {
  buildProjectFileBrowserRows,
  findProjectFileEntry,
  getProjectFileExtensionFromName,
  normalizeProjectRelativePath,
} from "./projectFileUtils";

export function ProjectLocalFilesPage({
  fileState,
  graphState,
  onLoadTextRange,
  onSelectFile,
}: {
  fileState: ProjectFilesState;
  graphState?: ProjectGraphState | null;
  onLoadTextRange?: (relativePath: string, startLine: number, lineCount: number) => Promise<ProjectFileTextRange>;
  onSelectFile: (relativePath: string) => void;
}) {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => new Set());
  const [directoryChildrenByPath, setDirectoryChildrenByPath] = useState<Record<string, ProjectFileBrowserRow["children"]>>({});
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
  const fileRows = useMemo(
    () => (filesSnapshot ? buildProjectFileBrowserRows(filesSnapshot.entries, expandedPaths, directoryChildrenByPath, content) : []),
    [content, directoryChildrenByPath, expandedPaths, filesSnapshot],
  );
  const recommendedFiles = useMemo(
    () => buildRecommendedFiles(graphState, filesSnapshot?.entries ?? null),
    [filesSnapshot?.entries, graphState],
  );
  const recommendedRows = useMemo(() => recommendedFiles.map(recommendedFileToRow), [recommendedFiles]);

  useEffect(() => {
    if (content?.kind !== "directory") {
      return;
    }
    setDirectoryChildrenByPath((current) => ({
      ...current,
      [content.relativePath]: content.directoryChildren,
    }));
  }, [content]);

  function handleProjectFileRowSelect(row: ProjectFileBrowserRow) {
    if (row.missing) {
      setRecommendedFileWarning("推荐文件已不存在，可能是 Graph 还未刷新。");
      return;
    }

    setRecommendedFileWarning(null);
    if (row.kind === "directory") {
      setExpandedPaths((current) => {
        const next = new Set(current);
        if (next.has(row.relativePath)) {
          next.delete(row.relativePath);
        } else {
          next.add(row.relativePath);
        }
        return next;
      });
    }
    onSelectFile(row.relativePath);
  }

  return (
    <section className="project-file-page" aria-label="项目本地文件阅读器">
      <article className="project-file-reader" aria-label="文件内容阅读器">
        <ProjectFileReader content={content} entry={selectedEntry} error={fileState.error} onLoadTextRange={onLoadTextRange} />
      </article>

      <ProjectFileBrowser
        expandedPaths={expandedPaths}
        onSelectRow={handleProjectFileRowSelect}
        recommendedFileWarning={recommendedFileWarning}
        recommendedRows={recommendedRows}
        rows={fileRows}
        selectedPath={selectedPath}
      />
    </section>
  );
}

function buildRecommendedFiles(graphState: ProjectGraphState | null | undefined, entries: readonly ProjectFileEntry[] | null) {
  const recommendedFiles: ProjectRecommendedFile[] = [];
  const seenPaths = new Set<string>();
  const projectEntries = entries ? [...entries] : [];

  function pushRecommendedFile(
    path: string,
    source: ProjectRecommendedFile["source"],
    reason: string,
  ) {
    const normalizedPath = normalizeProjectRelativePath(path);
    if (!normalizedPath || seenPaths.has(normalizedPath)) {
      return;
    }

    seenPaths.add(normalizedPath);
    const existingEntry = findProjectFileEntry(projectEntries, normalizedPath);
    recommendedFiles.push({
      path: normalizedPath,
      name: normalizedPath.split("/").filter(Boolean).at(-1) ?? normalizedPath,
      source,
      reason,
      status: existingEntry ? "available" : recommendedPathHasKnownAncestor(projectEntries, normalizedPath) ? "unloaded" : "missing",
    });
  }

  graphState?.latestContextPack?.recommendedFiles.forEach((file) => {
    pushRecommendedFile(file.path, "context-pack-file", file.reason || "Graph Context Pack 推荐文件");
  });
  graphState?.latestContextPack?.recommendedTests.forEach((file) => {
    pushRecommendedFile(file.path, "context-pack-test", file.reason || "Graph Context Pack 推荐测试");
  });
  graphState?.manifest?.importantFiles.forEach((path) => {
    pushRecommendedFile(path, "manifest-important", "Graph Manifest important file");
  });

  return recommendedFiles;
}

function recommendedPathHasKnownAncestor(entries: readonly ProjectFileEntry[], relativePath: string) {
  if (entries.length === 0) {
    return true;
  }

  const pathParts = relativePath.split("/").filter(Boolean);
  if (pathParts.length <= 1) {
    return false;
  }

  return entries.some((entry) => entry.kind === "directory" && normalizeProjectRelativePath(entry.relativePath) === pathParts[0]);
}

function recommendedFileToRow(recommendedFile: ProjectRecommendedFile): ProjectFileBrowserRow {
  return {
    name: recommendedFile.name,
    relativePath: recommendedFile.path,
    kind: "file",
    createdAt: null,
    modifiedAt: null,
    sizeBytes: null,
    extension: getProjectFileExtensionFromName(recommendedFile.name),
    childCount: null,
    isSymlink: false,
    missing: recommendedFile.status === "missing",
    recommendation: recommendedFile,
    children: [],
    depth: 0,
  };
}
