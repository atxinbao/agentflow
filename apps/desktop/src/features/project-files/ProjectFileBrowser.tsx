import { ChevronRight } from "lucide-react";
import type { CSSProperties } from "react";
import type { ProjectFileBrowserRow } from "./projectFileTypes";
import {
  formatProjectFileRowMeta,
  getProjectFileIconForNode,
  getProjectFileToneForNode,
  isHiddenProjectFilePath,
} from "./projectFileUtils";

export function ProjectFileBrowser({
  expandedPaths,
  onSelectRow,
  recommendedFileWarning,
  recommendedRows,
  rows,
  selectedPath,
}: {
  expandedPaths: ReadonlySet<string>;
  onSelectRow: (row: ProjectFileBrowserRow) => void;
  recommendedFileWarning?: string | null;
  recommendedRows?: ProjectFileBrowserRow[];
  rows: ProjectFileBrowserRow[];
  selectedPath: string | null;
}) {
  const hasRecommendedRows = Boolean(recommendedRows?.length);

  return (
    <aside className="project-file-browser" aria-label="项目文件列表">
      {hasRecommendedRows ? (
        <section className="project-recommended-files" aria-label="推荐文件">
          <header>
            <div>
              <h3>推荐文件</h3>
              <span>来自代码地图</span>
            </div>
          </header>
          <div className="project-recommended-file-list">
            {recommendedRows?.map((row) => (
              <button
                className={[
                  "project-recommended-file-chip",
                  row.missing ? "missing" : "",
                  selectedPath === row.relativePath ? "active" : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                key={`recommended-${row.relativePath}-${row.recommendation?.source ?? "unknown"}`}
                onClick={() => onSelectRow(row)}
                title={row.recommendation?.reason ?? row.relativePath}
                type="button"
              >
                <strong>{row.name}</strong>
                <span>{recommendedSourceLabel(row.recommendation?.source)}</span>
                <em>{recommendedStatusLabel(row.recommendation?.status)}</em>
              </button>
            ))}
          </div>
          {recommendedFileWarning ? <p>{recommendedFileWarning}</p> : null}
        </section>
      ) : null}
      <div className="project-file-table" role="table" aria-label="本地文件列表">
        <div className="project-file-table-head" role="row">
          <span role="columnheader">名称</span>
          <span role="columnheader">修改日期</span>
        </div>
        <div className="project-file-table-body">
          {rows.map((row) => {
            const isDirectory = row.kind === "directory";
            const isHidden = isHiddenProjectFilePath(row.relativePath);
            const fileTone = getProjectFileToneForNode(row.name, row.extension, row.kind, isHidden);
            const FileKindIcon = getProjectFileIconForNode(row.name, row.extension, row.kind, isHidden);
            const isExpanded = isDirectory && expandedPaths.has(row.relativePath);
            const rowClassName = [
              "project-file-row",
              isDirectory ? "directory" : "file",
              isHidden ? "hidden" : "visible",
              fileTone,
              row.depth > 0 ? "nested" : "",
              isExpanded ? "expanded" : "",
              selectedPath === row.relativePath ? "active" : "",
              row.missing ? "missing" : "",
            ]
              .filter(Boolean)
              .join(" ");

            return (
              <button
                className={rowClassName}
                key={row.relativePath}
                onClick={() => onSelectRow(row)}
                aria-expanded={isDirectory ? isExpanded : undefined}
                data-kind={row.kind}
                data-path={row.relativePath}
                role="row"
                style={{ "--project-file-depth": row.depth } as CSSProperties}
                type="button"
              >
                <span className="project-file-name" role="cell">
                  <span className="project-file-expander" aria-hidden="true">
                    {isDirectory ? <ChevronRight size={13} strokeWidth={2.2} /> : null}
                  </span>
                  <span className="project-file-kind-icon" aria-hidden="true">
                    <FileKindIcon size={15} strokeWidth={isHidden ? 2.3 : 2} />
                  </span>
                  <span className="project-file-label">{row.name}</span>
                </span>
                <span className="project-file-row-meta" role="cell">
                  {formatProjectFileRowMeta(row)}
                </span>
              </button>
            );
          })}
        </div>
      </div>
      <section className="project-readonly-note">
        <h3>只读展示</h3>
        <p>不执行/不写入，点击右侧任意文件或文件夹后，主体区域加载对应内容或目录概览。</p>
      </section>
    </aside>
  );
}

function recommendedSourceLabel(source?: NonNullable<ProjectFileBrowserRow["recommendation"]>["source"]) {
  if (source === "context-pack-file") return "Context";
  if (source === "context-pack-test") return "Test";
  if (source === "manifest-important") return "Important";
  return "Graph";
}

function recommendedStatusLabel(status?: NonNullable<ProjectFileBrowserRow["recommendation"]>["status"]) {
  if (status === "available") return "可打开";
  if (status === "missing") return "已不存在";
  if (status === "unloaded") return "未加载";
  return "未记录";
}
