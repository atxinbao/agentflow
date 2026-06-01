export type GraphStatus = "missing" | "indexing" | "ready" | "stale" | "failed" | "degraded";

export type GraphStatusSnapshot = {
  version: string;
  projectRoot: string;
  status: GraphStatus;
  fileCount: number;
  symbolCount: number;
  relationCount: number;
  updatedAt?: number | null;
  lastError?: string | null;
  watcherStatus?: string | null;
  watcherBackend?: string | null;
  preflightStatus?: string | null;
  protectionStatus?: string | null;
  degradedReasons?: string[];
};

export type GraphManifestSnapshot = {
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

export type GraphSearchSnapshot = {
  version: string;
  query: string;
  results: GraphSearchResult[];
};

export type GraphSearchResult = {
  kind: string;
  path: string;
  title: string;
  language?: string | null;
  symbolKind?: string | null;
  line?: number | null;
  snippet?: string | null;
  score: number;
};

export type GraphContextPack = {
  version: string;
  targetType: string;
  targetId?: string | null;
  query: string;
  createdAt: number;
  graphRevision?: string | null;
  recommendedFiles: GraphContextFile[];
  recommendedSymbols: GraphContextSymbol[];
  recommendedTests: GraphContextFile[];
  impactHints: GraphContextHint[];
  testHints: GraphTestHint[];
  confidence: string;
};

export type GraphContextFile = {
  path: string;
  reason: string;
  score: number;
};

export type GraphContextSymbol = {
  name: string;
  kind: string;
  path: string;
  line: number;
  score: number;
};

export type GraphContextHint = {
  path: string;
  reason: string;
  confidence: string;
};

export type GraphTestHint = {
  commandHint: string;
  reason: string;
  confidence: string;
  scope?: string;
};
