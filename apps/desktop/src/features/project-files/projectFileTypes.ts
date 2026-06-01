import type {
  ProjectDirectoryPage,
  ProjectFileChild,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileTextRange,
  ProjectFileSearchSnapshot,
  ProjectFileViewMode,
  ProjectFilesSnapshot,
} from "../../types";

export type ProjectFilesState = {
  snapshot: ProjectFilesSnapshot | null;
  content: ProjectFileContent | null;
  selectedPath: string | null;
  error: string | null;
  source: "tauri" | "preview" | "unavailable" | "loading";
  viewMode: ProjectFileViewMode;
  loading: boolean;
  loadingPath: string | null;
  directoryPages: Record<string, ProjectDirectoryPage>;
  searchQuery: string;
  searchSnapshot: ProjectFileSearchSnapshot | null;
  searchLoading: boolean;
  recentPaths: string[];
};

export type ProjectFileBrowserRow = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  extension?: string | null;
  sizeBytes?: number | null;
  childCount?: number | null;
  isSymlink?: boolean;
  children: ProjectFileChild[];
  depth: number;
  hasMoreChildren?: boolean;
  nextCursor?: string | null;
  totalChildren?: number | null;
  missing?: boolean;
};

export type {
  ProjectDirectoryPage,
  ProjectFileChild,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileSearchSnapshot,
  ProjectFileTextRange,
  ProjectFileViewMode,
};
