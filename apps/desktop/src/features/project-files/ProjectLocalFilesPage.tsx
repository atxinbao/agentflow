import { useEffect, useMemo, useState } from "react";
import { ProjectFileBrowser } from "./ProjectFileBrowser";
import { ProjectFileReader } from "./ProjectFileReader";
import "./ProjectFiles.css";
import type { ProjectFileBrowserRow, ProjectFilesState } from "./projectFileTypes";
import { buildProjectFileBrowserRows, findProjectFileEntry } from "./projectFileUtils";

export function ProjectLocalFilesPage({
  fileState,
  onSelectFile,
}: {
  fileState: ProjectFilesState;
  onSelectFile: (relativePath: string) => void;
}) {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => new Set());
  const [directoryChildrenByPath, setDirectoryChildrenByPath] = useState<Record<string, ProjectFileBrowserRow["children"]>>({});
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
        <ProjectFileReader content={content} entry={selectedEntry} error={fileState.error} />
      </article>

      <ProjectFileBrowser
        expandedPaths={expandedPaths}
        onSelectRow={handleProjectFileRowSelect}
        rows={fileRows}
        selectedPath={selectedPath}
      />
    </section>
  );
}
