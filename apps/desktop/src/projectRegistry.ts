export type AgentFlowProjectPage = "home" | "spec" | "tasks" | "audit" | "files" | "advanced";

export type AgentFlowProjectStatus = "ready" | "loading" | "blocked" | "error" | "missing";

export type AgentFlowProjectRef = {
  id: string;
  name: string;
  root: string;
  kind: "local";
  status: AgentFlowProjectStatus;
  lastOpenedAt: number;
  createdAt: number;
  expanded: boolean;
  lastActivePage: AgentFlowProjectPage;
  error?: string | null;
};

export type AgentFlowProjectRegistry = {
  activePageByProject: Record<string, AgentFlowProjectPage>;
  activeProjectRoot: string | null;
  expandedProjectRoots: Set<string>;
  projects: AgentFlowProjectRef[];
};

export const projectRegistryStorageKeys = {
  activePageByProject: "agentflow.activePageByProject.v1",
  activeProjectRoot: "agentflow.activeProjectRoot.v1",
  expandedProjectRoots: "agentflow.expandedProjectRoots.v1",
  projects: "agentflow.projects.v1",
} as const;

const projectPages: AgentFlowProjectPage[] = ["home", "spec", "tasks", "audit", "files", "advanced"];

export function isAgentFlowProjectPage(value: unknown): value is AgentFlowProjectPage {
  return typeof value === "string" && projectPages.includes(value as AgentFlowProjectPage);
}

function normalizeProjectPage(value: unknown): AgentFlowProjectPage | null {
  if (isAgentFlowProjectPage(value)) {
    return value;
  }
  return value === "execute" || value === "delivery" ? "tasks" : null;
}

export function stableProjectId(root: string) {
  let hash = 0;
  for (let index = 0; index < root.length; index += 1) {
    hash = (hash * 31 + root.charCodeAt(index)) >>> 0;
  }
  return `project-${hash.toString(16)}`;
}

export function createProjectRef({
  createdAt,
  expanded = true,
  lastActivePage = "home",
  name,
  root,
  status = "ready",
}: {
  createdAt?: number;
  expanded?: boolean;
  lastActivePage?: AgentFlowProjectPage;
  name: string;
  root: string;
  status?: AgentFlowProjectStatus;
}): AgentFlowProjectRef {
  const timestamp = createdAt ?? Date.now();
  return {
    createdAt: timestamp,
    error: null,
    expanded,
    id: stableProjectId(root),
    kind: "local",
    lastActivePage,
    lastOpenedAt: timestamp,
    name,
    root,
    status,
  };
}

export function readProjectRegistry({
  legacyActivePage,
  legacyProjectRoot,
  projectNameFromRoot,
}: {
  legacyActivePage: AgentFlowProjectPage;
  legacyProjectRoot: string | null;
  projectNameFromRoot: (root: string) => string;
}): AgentFlowProjectRegistry {
  const hasStoredProjectsKey = window.localStorage.getItem(projectRegistryStorageKeys.projects) !== null;
  const storedProjects = readProjects();
  const migratedProjects =
    hasStoredProjectsKey || !legacyProjectRoot
      ? storedProjects
      : [
          createProjectRef({
            lastActivePage: legacyActivePage,
            name: projectNameFromRoot(legacyProjectRoot),
            root: legacyProjectRoot,
          }),
        ];

  const activePageByProject = readActivePageByProject();
  const expandedProjectRoots = new Set(readStringArray(projectRegistryStorageKeys.expandedProjectRoots));
  let activeProjectRoot = window.localStorage.getItem(projectRegistryStorageKeys.activeProjectRoot) || null;
  const activeProjectExists = activeProjectRoot
    ? migratedProjects.some((project) => project.root === activeProjectRoot)
    : false;

  if (!activeProjectExists) {
    activeProjectRoot =
      migratedProjects.find((project) => project.root === legacyProjectRoot)?.root ??
      [...migratedProjects].sort((left, right) => right.lastOpenedAt - left.lastOpenedAt)[0]?.root ??
      null;
  }

  migratedProjects.forEach((project) => {
    if (project.expanded) {
      expandedProjectRoots.add(project.root);
    }
    activePageByProject[project.root] = activePageByProject[project.root] ?? project.lastActivePage;
  });

  return {
    activePageByProject,
    activeProjectRoot,
    expandedProjectRoots,
    projects: sortProjects(migratedProjects),
  };
}

export function createBrowserPreviewProjectRegistry(activeRoot: string): AgentFlowProjectRegistry {
  const timestamp = Date.now();
  const projects = [
    createProjectRef({
      createdAt: timestamp - 3000,
      expanded: true,
      lastActivePage: "home",
      name: "my-web-app",
      root: "/Users/mac/Documents/my-web-app",
      status: "ready",
    }),
    createProjectRef({
      createdAt: timestamp - 2000,
      expanded: false,
      lastActivePage: "tasks",
      name: "AgentFlow",
      root: activeRoot,
      status: "ready",
    }),
    createProjectRef({
      createdAt: timestamp - 1000,
      expanded: false,
      lastActivePage: "files",
      name: "mobile-app",
      root: "/Users/mac/Documents/mobile-app",
      status: "missing",
    }),
  ];

  const activePageByProject = Object.fromEntries(
    projects.map((project) => [project.root, project.lastActivePage]),
  ) as Record<string, AgentFlowProjectPage>;

  return {
    activePageByProject,
    activeProjectRoot: projects[0].root,
    expandedProjectRoots: new Set(projects.filter((project) => project.expanded).map((project) => project.root)),
    projects,
  };
}

export function persistProjectRegistry(registry: AgentFlowProjectRegistry) {
  window.localStorage.setItem(projectRegistryStorageKeys.projects, JSON.stringify(registry.projects));
  if (registry.activeProjectRoot) {
    window.localStorage.setItem(projectRegistryStorageKeys.activeProjectRoot, registry.activeProjectRoot);
  } else {
    window.localStorage.removeItem(projectRegistryStorageKeys.activeProjectRoot);
  }
  window.localStorage.setItem(
    projectRegistryStorageKeys.expandedProjectRoots,
    JSON.stringify([...registry.expandedProjectRoots]),
  );
  window.localStorage.setItem(
    projectRegistryStorageKeys.activePageByProject,
    JSON.stringify(registry.activePageByProject),
  );
}

export function upsertProject(
  registry: AgentFlowProjectRegistry,
  project: AgentFlowProjectRef,
): AgentFlowProjectRegistry {
  const existing = registry.projects.find((item) => item.root === project.root);
  const nextProject: AgentFlowProjectRef = existing
    ? {
        ...existing,
        ...project,
        createdAt: existing.createdAt,
        lastOpenedAt: Date.now(),
      }
    : {
        ...project,
        lastOpenedAt: Date.now(),
      };
  const projects = sortProjects([
    ...registry.projects.filter((item) => item.root !== project.root),
    nextProject,
  ]);
  const expandedProjectRoots = new Set(registry.expandedProjectRoots);
  if (nextProject.expanded) {
    expandedProjectRoots.add(nextProject.root);
  }

  return {
    activePageByProject: {
      ...registry.activePageByProject,
      [nextProject.root]: nextProject.lastActivePage,
    },
    activeProjectRoot: nextProject.root,
    expandedProjectRoots,
    projects,
  };
}

export function selectProject(registry: AgentFlowProjectRegistry, projectRoot: string): AgentFlowProjectRegistry {
  const activePage = registry.activePageByProject[projectRoot] ?? "home";
  return {
    ...registry,
    activePageByProject: {
      ...registry.activePageByProject,
      [projectRoot]: activePage,
    },
    activeProjectRoot: projectRoot,
    projects: registry.projects.map((project) =>
      project.root === projectRoot ? { ...project, lastActivePage: activePage, lastOpenedAt: Date.now() } : project,
    ),
  };
}

export function toggleProjectExpanded(
  registry: AgentFlowProjectRegistry,
  projectRoot: string,
): AgentFlowProjectRegistry {
  const expandedProjectRoots = new Set(registry.expandedProjectRoots);
  if (expandedProjectRoots.has(projectRoot)) {
    expandedProjectRoots.delete(projectRoot);
  } else {
    expandedProjectRoots.add(projectRoot);
  }

  return {
    ...registry,
    expandedProjectRoots,
    projects: registry.projects.map((project) =>
      project.root === projectRoot ? { ...project, expanded: expandedProjectRoots.has(projectRoot) } : project,
    ),
  };
}

export function removeProject(registry: AgentFlowProjectRegistry, projectRoot: string): AgentFlowProjectRegistry {
  const projects = registry.projects.filter((project) => project.root !== projectRoot);
  const expandedProjectRoots = new Set(registry.expandedProjectRoots);
  expandedProjectRoots.delete(projectRoot);
  const activePageByProject = projects.length
    ? Object.fromEntries(
        Object.entries(registry.activePageByProject).filter(([root]) =>
          projects.some((project) => project.root === root),
        ),
      )
    : {};
  const activeProjectRoot =
    registry.activeProjectRoot === projectRoot
      ? mostRecentlyOpenedProject(projects)?.root ?? null
      : registry.activeProjectRoot;
  if (activeProjectRoot && registry.activeProjectRoot === projectRoot) {
    expandedProjectRoots.add(activeProjectRoot);
  }

  return {
    activePageByProject,
    activeProjectRoot,
    expandedProjectRoots,
    projects,
  };
}

export function setProjectPage(
  registry: AgentFlowProjectRegistry,
  projectRoot: string | null,
  page: AgentFlowProjectPage,
): AgentFlowProjectRegistry {
  if (!projectRoot) {
    return registry;
  }

  return {
    ...registry,
    activePageByProject: {
      ...registry.activePageByProject,
      [projectRoot]: page,
    },
    activeProjectRoot: projectRoot,
    projects: registry.projects.map((project) =>
      project.root === projectRoot ? { ...project, lastActivePage: page, lastOpenedAt: Date.now() } : project,
    ),
  };
}

function readProjects(): AgentFlowProjectRef[] {
  const value = readJson(projectRegistryStorageKeys.projects);
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter(isProjectRef).map((project) => ({
    ...project,
    lastActivePage: normalizeProjectPage(project.lastActivePage) ?? "home",
    status: project.status === "loading" ? "ready" : project.status,
  }));
}

function readActivePageByProject(): Record<string, AgentFlowProjectPage> {
  const value = readJson(projectRegistryStorageKeys.activePageByProject);
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return {};
  }

  return Object.fromEntries(
    Object.entries(value)
      .map(([root, page]) => [root, normalizeProjectPage(page)] as const)
      .filter((entry): entry is [string, AgentFlowProjectPage] => entry[1] !== null),
  );
}

function readStringArray(key: string) {
  const value = readJson(key);
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}

function readJson(key: string): unknown {
  try {
    return JSON.parse(window.localStorage.getItem(key) ?? "null");
  } catch {
    return null;
  }
}

function isProjectRef(value: unknown): value is AgentFlowProjectRef {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const candidate = value as Partial<AgentFlowProjectRef>;
  return (
    typeof candidate.id === "string" &&
    typeof candidate.name === "string" &&
    typeof candidate.root === "string" &&
    candidate.kind === "local" &&
    typeof candidate.lastOpenedAt === "number" &&
    typeof candidate.createdAt === "number"
  );
}

function sortProjects(projects: AgentFlowProjectRef[]) {
  return [...projects].sort((left, right) => right.lastOpenedAt - left.lastOpenedAt);
}

function mostRecentlyOpenedProject(projects: AgentFlowProjectRef[]) {
  return [...projects].sort((left, right) => right.lastOpenedAt - left.lastOpenedAt)[0] ?? null;
}
