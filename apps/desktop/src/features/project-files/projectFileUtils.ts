import { File as FileIcon, FileBadge, FileCode, FileJson, FileText, Folder, FolderDot, type LucideIcon } from "lucide-react";
import type { ProjectFileChild, ProjectFileContent, ProjectFileEntry } from "../../types";
import type { ProjectFileBrowserRow } from "./projectFileTypes";

export function isProjectCodeLanguage(language: string, name: string) {
  return (
    ["json", "toml", "yaml", "rust", "typescript", "javascript", "css", "html", "shell", "config"].includes(language) ||
    [".json", ".toml", ".yaml", ".yml", ".rs", ".ts", ".tsx", ".js", ".jsx", ".css", ".html", ".sh", ".gitignore"].some(
      (suffix) => name.endsWith(suffix),
    )
  );
}

export function findProjectFileEntry(entries: ProjectFileEntry[], relativePath: string): ProjectFileEntry | null {
  const targetPath = normalizeProjectRelativePath(relativePath);
  for (const entry of entries) {
    if (normalizeProjectRelativePath(entry.relativePath) === targetPath) {
      return entry;
    }

    const childEntry: ProjectFileEntry | null = findProjectFileEntry(entry.children.map(projectFileChildToEntry), targetPath);
    if (childEntry) {
      return childEntry;
    }
  }

  return null;
}

export function buildProjectFileBrowserRows(
  entries: ProjectFileEntry[],
  expandedPaths: ReadonlySet<string>,
  directoryChildrenByPath: Readonly<Record<string, ProjectFileChild[]>>,
  activeContent: ProjectFileContent | null,
) {
  const rows: ProjectFileBrowserRow[] = [];

  function appendRow(row: ProjectFileBrowserRow) {
    rows.push(row);
    if (row.kind !== "directory" || !expandedPaths.has(row.relativePath)) {
      return;
    }

    const children = getProjectFileBrowserChildren(row, directoryChildrenByPath, activeContent);
    children.forEach((child) => {
      const childPath = normalizeProjectRelativePath(child.relativePath);
      appendRow({
        name: child.name,
        relativePath: childPath,
        kind: child.kind,
        createdAt: null,
        modifiedAt: null,
        sizeBytes: null,
        extension: getProjectFileExtensionFromName(child.name),
        childCount: null,
        isSymlink: false,
        children: [],
        depth: row.depth + 1,
      });
    });
  }

  entries.forEach((entry) => {
    appendRow({
      name: entry.name,
      relativePath: entry.relativePath,
      kind: entry.kind,
      createdAt: entry.createdAt,
      modifiedAt: entry.modifiedAt,
      sizeBytes: entry.sizeBytes,
      extension: entry.extension,
      childCount: entry.childCount,
      isSymlink: false,
      children: entry.children,
      depth: 0,
    });
  });

  return rows;
}

function getProjectFileBrowserChildren(
  row: ProjectFileBrowserRow,
  directoryChildrenByPath: Readonly<Record<string, ProjectFileChild[]>>,
  activeContent: ProjectFileContent | null,
) {
  if (activeContent?.kind === "directory" && activeContent.relativePath === row.relativePath) {
    return activeContent.directoryChildren;
  }
  return directoryChildrenByPath[row.relativePath] ?? row.children;
}

export function formatProjectFileRowMeta(row: ProjectFileBrowserRow) {
  if (row.missing) {
    return "已不存在";
  }
  if (row.modifiedAt) {
    return formatProjectFileTimestamp(row.modifiedAt);
  }
  return row.kind === "directory" ? "文件夹" : "文件";
}

export function isHiddenProjectFileEntry(entry: ProjectFileEntry) {
  return isHiddenProjectFilePath(entry.relativePath);
}

export function isHiddenProjectFilePath(relativePath: string) {
  return relativePath.split("/").some((part) => part.startsWith("."));
}

export function getProjectFileTone(entry: ProjectFileEntry, isHidden: boolean) {
  return getProjectFileToneForNode(entry.name, entry.extension, entry.kind, isHidden);
}

export function getProjectFileToneForNode(name: string, extension: string | null | undefined, kind: "file" | "directory", isHidden: boolean) {
  if (kind === "directory") {
    return isHidden ? "hidden-directory" : "visible-directory";
  }

  return getProjectFileToneFromName(name, extension ?? getProjectFileExtensionFromName(name), isHidden);
}

export function getProjectFileToneFromName(name: string, extension: string, isHidden: boolean) {
  if (isHidden) {
    return "hidden-file";
  }

  const normalizedExtension = extension.toLowerCase();
  const normalizedName = name.toLowerCase();

  if (["md", "mdx", "markdown", "txt", "rst", "adoc"].includes(normalizedExtension)) {
    return "document-file";
  }

  if (["json", "jsonc"].includes(normalizedExtension)) {
    return "data-file";
  }

  if (
    ["toml", "yaml", "yml", "lock", "rs", "ts", "tsx", "js", "jsx", "css", "html", "sh"].includes(normalizedExtension) ||
    ["cargo.lock", "makefile", "dockerfile"].includes(normalizedName)
  ) {
    return "code-file";
  }

  return "plain-file";
}

export function getProjectFileIcon(entry: ProjectFileEntry, isHidden: boolean): LucideIcon {
  return getProjectFileIconForNode(entry.name, entry.extension, entry.kind, isHidden);
}

export function getProjectFileIconForNode(
  name: string,
  extension: string | null | undefined,
  kind: "file" | "directory",
  isHidden: boolean,
): LucideIcon {
  if (kind === "directory") {
    return isHidden ? FolderDot : Folder;
  }

  return getProjectFileIconFromTone(getProjectFileToneForNode(name, extension, kind, isHidden));
}

export function getProjectFileIconFromTone(tone: string): LucideIcon {
  if (tone === "hidden-file") {
    return FileBadge;
  }
  if (tone === "document-file") {
    return FileText;
  }
  if (tone === "data-file") {
    return FileJson;
  }
  if (tone === "code-file") {
    return FileCode;
  }

  return FileIcon;
}

export function getProjectFileExtensionFromName(name: string) {
  const lastDotIndex = name.lastIndexOf(".");
  if (lastDotIndex <= 0 || lastDotIndex === name.length - 1) {
    return "";
  }
  return name.slice(lastDotIndex + 1);
}

export function formatProjectFileTimestamp(value?: number | null) {
  if (!value) {
    return "未记录";
  }
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

export function formatProjectFileSize(value?: number | null) {
  if (value === null || value === undefined) {
    return "未记录";
  }
  if (value <= 0) {
    return "0 KB";
  }
  if (value < 1024) {
    return "1 KB";
  }
  if (value < 1024 * 1024) {
    return `${Math.max(1, Math.round(value / 1024))} KB`;
  }
  return `${(value / 1024 / 1024).toFixed(value < 10 * 1024 * 1024 ? 1 : 0)} MB`;
}

export function projectFileChildToEntry(child: ProjectFileChild): ProjectFileEntry {
  const enrichedChild = child as ProjectFileChild & Partial<ProjectFileEntry>;
  return {
    name: child.name,
    relativePath: normalizeProjectRelativePath(child.relativePath),
    kind: child.kind,
    createdAt: enrichedChild.createdAt ?? null,
    modifiedAt: enrichedChild.modifiedAt ?? null,
    sizeBytes: enrichedChild.sizeBytes ?? null,
    extension: enrichedChild.extension ?? (child.kind === "file" ? getProjectFileExtensionFromName(child.name) : null),
    childCount: enrichedChild.childCount ?? (child.kind === "directory" ? 0 : null),
    children: enrichedChild.children ?? [],
  };
}

export function joinProjectRelativePath(parentPath: string, name: string) {
  const normalizedParent = normalizeProjectRelativePath(parentPath);
  return normalizedParent ? `${normalizedParent}/${name}` : name;
}

export function normalizeProjectRelativePath(relativePath: string) {
  return relativePath.replace(/^\/+/, "");
}

export function projectNameFromPath(projectRoot: string) {
  const normalized = normalizeProjectRootKey(projectRoot);
  return normalized.split(/[\\/]/).filter(Boolean).at(-1) ?? projectRoot;
}

export function normalizeProjectRootKey(projectRoot: string) {
  return projectRoot.trim().replace(/[\\/]+$/, "");
}

export function projectRootsEqual(leftRoot?: string | null, rightRoot?: string | null) {
  if (!leftRoot || !rightRoot) {
    return false;
  }
  return normalizeProjectRootKey(leftRoot) === normalizeProjectRootKey(rightRoot);
}
