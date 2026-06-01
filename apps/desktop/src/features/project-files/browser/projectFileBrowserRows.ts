import type { ProjectDirectoryPage, ProjectFileContent, ProjectFileEntry } from "../../../types";
import type { ProjectFileBrowserRow } from "../model/projectFileTypes";
import {
  getProjectFileExtensionFromName,
  normalizeProjectRelativePath,
} from "../model/projectFileUtils";

export function buildProjectFileBrowserRows(
  entries: ProjectFileEntry[],
  expandedPaths: ReadonlySet<string>,
  directoryPagesByPath: Readonly<Record<string, ProjectDirectoryPage>>,
  activeContent: ProjectFileContent | null,
) {
  const rows: ProjectFileBrowserRow[] = [];

  function appendRow(row: ProjectFileBrowserRow) {
    rows.push(row);
    if (row.kind !== "directory" || !expandedPaths.has(row.relativePath)) {
      return;
    }

    const directoryPage = directoryPagesByPath[row.relativePath];
    const children = getProjectFileBrowserChildren(row, directoryPagesByPath, activeContent);
    children.forEach((child) => {
      const childPath = normalizeProjectRelativePath(child.relativePath);
      appendRow({
        name: child.name,
        relativePath: childPath,
        kind: child.kind,
        createdAt: child.createdAt ?? null,
        modifiedAt: child.modifiedAt ?? null,
        sizeBytes: child.sizeBytes ?? null,
        childCount: child.childCount ?? null,
        isSymlink: child.isSymlink ?? false,
        extension: child.extension ?? getProjectFileExtensionFromName(child.name),
        children: [],
        depth: row.depth + 1,
      });
    });
    if (directoryPage?.nextCursor) {
      rows.push({
        name: "加载更多",
        relativePath: `${row.relativePath}::__load_more__`,
        kind: "file",
        createdAt: null,
        modifiedAt: null,
        sizeBytes: null,
        childCount: null,
        isSymlink: false,
        extension: null,
        children: [],
        depth: row.depth + 1,
        hasMoreChildren: true,
        nextCursor: directoryPage.nextCursor,
        totalChildren: directoryPage.totalChildren,
      });
    }
  }

  entries.forEach((entry) => {
    appendRow({
      name: entry.name,
      relativePath: entry.relativePath,
      kind: entry.kind,
      createdAt: entry.createdAt,
      modifiedAt: entry.modifiedAt,
      extension: entry.extension,
      sizeBytes: entry.sizeBytes,
      childCount: entry.childCount,
      isSymlink: entry.isSymlink ?? false,
      children: entry.children,
      depth: 0,
    });
  });

  return rows;
}

function getProjectFileBrowserChildren(
  row: ProjectFileBrowserRow,
  directoryPagesByPath: Readonly<Record<string, ProjectDirectoryPage>>,
  activeContent: ProjectFileContent | null,
) {
  if (directoryPagesByPath[row.relativePath]) {
    return directoryPagesByPath[row.relativePath].entries;
  }
  if (activeContent?.kind === "directory" && activeContent.relativePath === row.relativePath) {
    return activeContent.directoryChildren;
  }
  return row.children;
}
