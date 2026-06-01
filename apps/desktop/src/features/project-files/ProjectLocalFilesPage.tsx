import { useEffect, useMemo, useState } from "react";
import { ProjectFileBrowser } from "./ProjectFileBrowser";
import { ProjectFileReader } from "./ProjectFileReader";
import "./ProjectFiles.css";
import type { ProjectGraphState } from "./useProjectGraph";
import type { ProjectFileBrowserRow, ProjectFilesState } from "./projectFileTypes";
import { buildProjectFileBrowserRows, findProjectFileEntry } from "./projectFileUtils";

export function ProjectLocalFilesPage({
  fileState,
  graphState,
  onSelectFile,
}: {
  fileState: ProjectFilesState;
  graphState: ProjectGraphState;
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
      <div className="project-file-reader-column">
        <ProjectGraphSummary graphState={graphState} />
        <article className="project-file-reader" aria-label="文件内容阅读器">
          <ProjectFileReader content={content} entry={selectedEntry} error={fileState.error} />
        </article>
      </div>

      <ProjectFileBrowser
        expandedPaths={expandedPaths}
        onSelectRow={handleProjectFileRowSelect}
        rows={fileRows}
        selectedPath={selectedPath}
      />
    </section>
  );
}

function ProjectGraphSummary({ graphState }: { graphState: ProjectGraphState }) {
  const status = graphState.status;
  const manifest = graphState.manifest;
  const statusText = graphStatusLabel(status?.status ?? "missing");
  const languageText = manifest?.languages.length ? manifest.languages.slice(0, 5).join(" / ") : "未记录";

  return (
    <section className="project-graph-summary" aria-label="代码地图状态">
      <div>
        <span className="project-graph-kicker">代码地图</span>
        <strong>{statusText}</strong>
      </div>
      <dl>
        <div>
          <dt>文件</dt>
          <dd>{status?.fileCount ?? 0}</dd>
        </div>
        <div>
          <dt>符号</dt>
          <dd>{status?.symbolCount ?? 0}</dd>
        </div>
        <div>
          <dt>关系</dt>
          <dd>{status?.relationCount ?? 0}</dd>
        </div>
        <div>
          <dt>语言</dt>
          <dd>{languageText}</dd>
        </div>
      </dl>
      {graphState.error ? <p>{graphState.error}</p> : null}
    </section>
  );
}

function graphStatusLabel(status: string) {
  const labels: Record<string, string> = {
    missing: "未建立",
    indexing: "建立中",
    ready: "已就绪",
    stale: "需更新",
    failed: "失败",
    degraded: "降级",
  };
  return labels[status] ?? status;
}
