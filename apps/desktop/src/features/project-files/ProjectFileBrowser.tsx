import { ChevronRight, Loader2, Search } from "lucide-react";
import type { CSSProperties } from "react";
import type { ProjectFileBrowserRow, ProjectFileViewMode } from "./projectFileTypes";
import {
  formatProjectFileRowMeta,
  getProjectFileIconForNode,
  getProjectFileToneForNode,
  isHiddenProjectFilePath,
} from "./projectFileUtils";

const viewModeOptions: Array<{ value: ProjectFileViewMode; label: string }> = [
  { value: "source", label: "源码" },
  { value: "all", label: "全部" },
  { value: "recent", label: "最近" },
];

export function ProjectFileBrowser({
  expandedPaths,
  loading,
  onChangeViewMode,
  onSearchChange,
  onSelectRow,
  recommendedFilePaths,
  recommendedRows,
  rows,
  searchLoading,
  searchQuery,
  selectedPath,
  viewMode,
}: {
  expandedPaths: ReadonlySet<string>;
  loading: boolean;
  onChangeViewMode: (viewMode: ProjectFileViewMode) => void;
  onSearchChange: (query: string) => void;
  onSelectRow: (row: ProjectFileBrowserRow) => void;
  recommendedFilePaths: string[];
  recommendedRows: ProjectFileBrowserRow[];
  rows: ProjectFileBrowserRow[];
  searchLoading: boolean;
  searchQuery: string;
  selectedPath: string | null;
  viewMode: ProjectFileViewMode;
}) {
  return (
    <aside className="project-file-browser" aria-label="项目文件列表">
      <header className="project-file-browser-toolbar">
        <label className="project-file-search">
          <Search size={14} />
          <input
            aria-label="搜索项目文件"
            onChange={(event) => onSearchChange(event.currentTarget.value)}
            placeholder="搜索文件..."
            value={searchQuery}
          />
          {searchLoading ? <Loader2 className="spin" size={13} /> : null}
        </label>
        <div className="project-file-view-modes" aria-label="文件视图模式">
          {viewModeOptions.map((option) => (
            <button
              className={viewMode === option.value ? "active" : ""}
              key={option.value}
              onClick={() => onChangeViewMode(option.value)}
              type="button"
            >
              {option.label}
            </button>
          ))}
        </div>
      </header>
      <div className="project-file-table" role="table" aria-label="本地文件列表">
        {recommendedRows.length > 0 ? (
          <section className="project-file-recommended" aria-label="推荐文件">
            <div className="project-file-recommended-heading">
              <span>推荐文件</span>
              <small>来自代码地图</small>
            </div>
            <div className="project-file-recommended-list">
              {recommendedRows.map((row) => {
                const isMissing = Boolean(row.missing);
                const FileKindIcon = getProjectFileIconForNode(row.name, row.extension, row.kind, isHiddenProjectFilePath(row.relativePath));
                return (
                  <button
                    className={`project-file-recommended-chip${isMissing ? " missing" : ""}`}
                    key={row.relativePath}
                    onClick={() => onSelectRow(row)}
                    title={isMissing ? "推荐文件已不存在" : row.relativePath}
                    type="button"
                  >
                    <FileKindIcon size={14} />
                    <span>{row.name}</span>
                    {isMissing ? <small>已不存在</small> : null}
                  </button>
                );
              })}
            </div>
          </section>
        ) : null}
        <div className="project-file-table-head" role="row">
          <span role="columnheader">名称</span>
          <span role="columnheader">修改日期</span>
        </div>
        <div className="project-file-table-body">
          {rows.length === 0 ? (
            <div className="project-file-table-empty">
              {searchQuery.trim() ? "没有匹配的文件。" : viewMode === "recent" ? "暂无最近打开文件。" : "当前目录暂无文件。"}
            </div>
          ) : (
            rows.map((row) => {
              const isDirectory = row.kind === "directory";
              const isRecommended = recommendedFilePaths.includes(row.relativePath);
              const isHidden = isHiddenProjectFilePath(row.relativePath);
              const fileTone = getProjectFileToneForNode(row.name, row.extension, row.kind, isHidden);
              const FileKindIcon = getProjectFileIconForNode(row.name, row.extension, row.kind, isHidden);
              const isExpanded = isDirectory && expandedPaths.has(row.relativePath);
              const isLoadMore = Boolean(row.hasMoreChildren);
              const rowClassName = [
                "project-file-row",
                isDirectory ? "directory" : "file",
                isHidden ? "hidden" : "visible",
                fileTone,
                row.depth > 0 ? "nested" : "",
                isExpanded ? "expanded" : "",
                selectedPath === row.relativePath ? "active" : "",
                isLoadMore ? "load-more" : "",
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
                  disabled={loading && isLoadMore}
                  role="row"
                  style={{ "--project-file-depth": row.depth } as CSSProperties}
                  type="button"
                >
                  <span className="project-file-name" role="cell">
                    <span className="project-file-expander" aria-hidden="true">
                      {isDirectory ? <ChevronRight size={13} strokeWidth={2.2} /> : null}
                    </span>
                    <span className="project-file-kind-icon" aria-hidden="true">
                      {isLoadMore ? <Loader2 size={14} /> : <FileKindIcon size={15} strokeWidth={isHidden ? 2.3 : 2} />}
                    </span>
                    <span className="project-file-label">
                      {row.name}
                      {isRecommended ? <small>推荐</small> : null}
                      {row.isSymlink ? <small>链接</small> : null}
                    </span>
                  </span>
                  <span className="project-file-row-meta" role="cell">
                    {isLoadMore ? `共 ${row.totalChildren ?? "多"} 项` : formatProjectFileRowMeta(row)}
                  </span>
                </button>
              );
            })
          )}
        </div>
      </div>
      <section className="project-readonly-note">
        <h3>只读展示</h3>
        <p>点击右侧任意文件或文件夹后，主体区域加载对应内容或目录概览。</p>
        <p>不执行命令，不写入工作区。</p>
      </section>
    </aside>
  );
}
