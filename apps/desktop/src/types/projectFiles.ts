export type ProjectFilesSnapshot = {
  version: string;
  projectRoot: string;
  entries: ProjectFileEntry[];
  selectedPath?: string | null;
};

export type ProjectFileEntry = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  childCount?: number | null;
  isSymlink?: boolean;
  children: ProjectFileChild[];
};

export type ProjectFileChild = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  childCount?: number | null;
  isSymlink?: boolean;
};

export type ProjectFileContent = {
  relativePath: string;
  name: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  mimeType?: string | null;
  language: string;
  content?: string | null;
  binaryPreview?: string | null;
  dataUrl?: string | null;
  truncated?: boolean;
  directoryChildren: ProjectFileChild[];
  unsupportedReason?: string | null;
};

export type ProjectFileViewMode = "source" | "all" | "recent";

export type ProjectDirectoryPage = {
  version: string;
  projectRoot: string;
  directoryPath: string;
  entries: ProjectFileChild[];
  nextCursor?: string | null;
  totalChildren: number;
  limit: number;
  viewMode: ProjectFileViewMode | string;
};

export type ProjectFileSearchSnapshot = {
  version: string;
  projectRoot: string;
  query: string;
  viewMode: ProjectFileViewMode | string;
  results: ProjectFileSearchResult[];
};

export type ProjectFileSearchResult = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  extension?: string | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  score: number;
  matchReason: string;
};

export type ProjectFileTextRange = {
  version: string;
  projectRoot: string;
  relativePath: string;
  startLine: number;
  endLine: number;
  totalLines: number;
  content: string;
  truncated: boolean;
};
