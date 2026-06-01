import type {
  ProjectFileChild,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileTextRange,
  ProjectFilesSnapshot,
  ProjectRecommendedFile,
} from "../../types";

export type ProjectFilesState = {
  snapshot: ProjectFilesSnapshot | null;
  content: ProjectFileContent | null;
  selectedPath: string | null;
  error: string | null;
  source: "tauri" | "preview" | "unavailable" | "loading";
};

export type ProjectFileBrowserRow = {
  name: string;
  relativePath: string;
  kind: "file" | "directory";
  createdAt?: number | null;
  modifiedAt?: number | null;
  sizeBytes?: number | null;
  extension?: string | null;
  childCount?: number | null;
  isSymlink?: boolean;
  missing?: boolean;
  recommendation?: ProjectRecommendedFile;
  children: ProjectFileChild[];
  depth: number;
};

export type { ProjectFileChild, ProjectFileContent, ProjectFileEntry, ProjectFileTextRange, ProjectRecommendedFile };
