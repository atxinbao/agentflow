export type PanelStatus = "missing" | "indexing" | "ready" | "stale" | "failed";

export type PanelStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: PanelStatus;
  fileCount: number;
  symbolCount: number;
  relationCount: number;
  updatedAt?: number | null;
  lastError?: string | null;
  watcherStatus?: string | null;
  watcherBackend?: string | null;
  preflightStatus?: string | null;
  protectionStatus?: string | null;
  warnings?: string[];
};

export type PanelManifestSnapshot = {
  version: string;
  projectRoot: string;
  languages: string[];
  topLevelDirs: string[];
  importantFiles: string[];
  sourceFiles: number;
  testFiles: number;
  docFiles: number;
  configFiles: number;
  platforms?: string[];
  entryPoints?: string[];
  mobileComponents?: string[];
  mobileConfigs?: string[];
  mobileTests?: string[];
};

export type PanelSearchSnapshot = {
  version: string;
  query: string;
  results: PanelSearchResult[];
};

export type PanelSearchResult = {
  kind: string;
  path: string;
  title: string;
  language?: string | null;
  symbolKind?: string | null;
  line?: number | null;
  snippet?: string | null;
  score: number;
};

export type PanelContextPack = {
  version: string;
  targetType: string;
  targetId?: string | null;
  query: string;
  createdAt: number;
  panelRevision?: string | null;
  recommendedFiles: PanelContextFile[];
  recommendedSymbols: PanelContextSymbol[];
  recommendedTests: PanelContextFile[];
  impactHints: PanelContextHint[];
  testHints: PanelTestHint[];
  confidence: string;
};

export type PanelContextFile = {
  path: string;
  reason: string;
  score: number;
};

export type PanelContextSymbol = {
  name: string;
  kind: string;
  path: string;
  line: number;
  score: number;
};

export type PanelContextHint = {
  path: string;
  reason: string;
  confidence: string;
};

export type PanelTestHint = {
  commandHint: string;
  reason: string;
  confidence: string;
  scope?: string;
};
