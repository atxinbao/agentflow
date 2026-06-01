import { File as FileIcon, Folder } from "lucide-react";
import type { ProjectFileContent, ProjectFileEntry, ProjectFileTextRange } from "../../types";
import { ProjectFileBodyRenderer } from "./FileRendererRegistry";
import {
  formatProjectFileSize,
  formatProjectFileTimestamp,
  getProjectFileExtensionFromName,
  getProjectFileIcon,
  getProjectFileIconFromTone,
  getProjectFileTone,
  getProjectFileToneForNode,
  isHiddenProjectFileEntry,
} from "./projectFileUtils";

export function ProjectFileReader({
  content,
  entry,
  error,
  onLoadTextRange,
}: {
  content: ProjectFileContent | null;
  entry: ProjectFileEntry | null;
  error?: string | null;
  onLoadTextRange?: (relativePath: string, startLine: number, lineCount: number) => Promise<ProjectFileTextRange>;
}) {
  if (error && !content) {
    return (
      <section className="project-file-empty">
        <h3>无法加载项目文件</h3>
        <p>{error}</p>
      </section>
    );
  }

  if (!content && !entry) {
    return (
      <section className="project-file-empty">
        <h3>未选择文件</h3>
        <p>请在右侧文件列表中选择一个文件或文件夹。</p>
      </section>
    );
  }

  if (content?.kind === "directory" || entry?.kind === "directory") {
    const directoryName = content?.name ?? entry?.name ?? "目录";
    const children = content?.directoryChildren ?? entry?.children ?? [];
    return (
      <section className="project-directory-overview">
        <header>
          <span>目录概览</span>
          <h3>{directoryName}</h3>
          <p>{content?.relativePath ?? entry?.relativePath}</p>
        </header>
        <div className="project-directory-child-list">
          {children.length === 0 ? (
            <p className="empty">当前目录暂无可展示子项。</p>
          ) : (
            children.map((child) => (
              <div className="project-directory-child" key={child.relativePath}>
                {child.kind === "directory" ? <Folder size={15} /> : <FileIcon size={15} />}
                <span>{child.name}</span>
                <small>{child.kind === "directory" ? "文件夹" : "文件"}</small>
              </div>
            ))
          )}
        </div>
      </section>
    );
  }

  const language = content?.language ?? entry?.extension ?? "text";
  const title = content?.name ?? entry?.name ?? "文件";
  const displayType = projectFileDisplayType(content, title, language);
  const fileSize = formatProjectFileSize(content?.sizeBytes ?? entry?.sizeBytes);
  const createdAt = formatProjectFileTimestamp(content?.createdAt ?? entry?.createdAt);
  const isHidden = entry ? isHiddenProjectFileEntry(entry) : title.startsWith(".");
  const fileTone = entry
    ? getProjectFileTone(entry, isHidden)
    : getProjectFileToneForNode(title, content?.extension ?? getProjectFileExtensionFromName(title), "file", isHidden);
  const FileKindIcon = entry ? getProjectFileIcon(entry, isHidden) : getProjectFileIconFromTone(fileTone);

  return (
    <section className={`project-file-content ${fileTone}`}>
      <header className="project-file-content-header">
        <div className="project-file-title-group">
          <span className="project-file-title-icon" aria-hidden="true">
            <FileKindIcon size={18} strokeWidth={isHidden ? 2.3 : 2} />
          </span>
          <div>
            <span>{displayType}</span>
            <h3>{title}</h3>
          </div>
        </div>
        <dl aria-label="文件元数据">
          <div>
            <dt>文件大小</dt>
            <dd>{fileSize}</dd>
          </div>
          <div>
            <dt>创建日期</dt>
            <dd>{createdAt}</dd>
          </div>
        </dl>
      </header>
      {content ? <ProjectFileBodyRenderer content={content} entry={entry} onLoadTextRange={onLoadTextRange} /> : null}
    </section>
  );
}

function projectFileDisplayType(content: ProjectFileContent | null, title: string, language: string) {
  const extension = (content?.extension ?? getProjectFileExtensionFromName(title)).toLowerCase();
  const mimeType = content?.mimeType ?? "";
  if (language === "markdown" || ["md", "mdx", "markdown"].includes(extension)) return "Markdown 文档";
  if (mimeType.startsWith("image/")) return "Image";
  if (mimeType.startsWith("audio/") || mimeType.startsWith("video/")) return "Media";
  if (mimeType === "application/pdf" || extension === "pdf") return "PDF";
  if (extension === "docx") return "DOCX";
  if (["csv", "tsv", "xlsx"].includes(extension)) return "Table";
  if (language === "json" || extension === "json") return "JSON";
  if (content?.binaryPreview) return "Binary";
  return "Code";
}
