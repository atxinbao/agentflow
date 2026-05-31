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
  rows,
  selectedPath,
}: {
  expandedPaths: ReadonlySet<string>;
  onSelectRow: (row: ProjectFileBrowserRow) => void;
  rows: ProjectFileBrowserRow[];
  selectedPath: string | null;
}) {
  return (
    <aside className="project-file-browser" aria-label="项目文件列表">
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
