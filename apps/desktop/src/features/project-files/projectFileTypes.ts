import type { ProjectFileChild, ProjectFileContent, ProjectFileEntry, ProjectFilesSnapshot } from "../../types";

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
  extension?: string | null;
  children: ProjectFileChild[];
  depth: number;
};

export type { ProjectFileChild, ProjectFileContent, ProjectFileEntry };
