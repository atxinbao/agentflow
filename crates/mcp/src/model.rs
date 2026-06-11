use serde::{Deserialize, Serialize};

pub const MCP_PROVIDER_STATUS_VERSION: &str = "agentflow-mcp-provider.v1";
pub const MCP_REGISTRY_VERSION: &str = "agentflow-mcp-registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProviderKind {
    Github,
    Gitlab,
    Codex,
    BrowserPreview,
}

impl McpProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Codex => "codex",
            Self::BrowserPreview => "browser-preview",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProviderStatusCode {
    Ready,
    Unavailable,
    Unauthenticated,
    PermissionDenied,
    Unsupported,
    Failed,
}

impl McpProviderStatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
            Self::Unauthenticated => "unauthenticated",
            Self::PermissionDenied => "permission-denied",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCapability {
    pub name: String,
    pub available: bool,
    pub detail: Option<String>,
}

impl McpCapability {
    pub fn new(name: impl Into<String>, available: bool) -> Self {
        Self {
            name: name.into(),
            available,
            detail: None,
        }
    }

    pub fn with_detail(
        name: impl Into<String>,
        available: bool,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            available,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpProviderStatus {
    pub version: String,
    pub provider: String,
    pub kind: McpProviderKind,
    pub status: McpProviderStatusCode,
    pub capabilities: Vec<McpCapability>,
    pub cli: Option<String>,
    pub installed: bool,
    pub authenticated: Option<bool>,
    pub repo_permission_checked: bool,
    pub repo_permission: Option<String>,
    pub checked_at: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl McpProviderStatus {
    pub fn new(kind: McpProviderKind, checked_at: u64) -> Self {
        Self {
            version: MCP_PROVIDER_STATUS_VERSION.to_string(),
            provider: kind.as_str().to_string(),
            kind,
            status: McpProviderStatusCode::Unavailable,
            capabilities: Vec::new(),
            cli: None,
            installed: false,
            authenticated: None,
            repo_permission_checked: false,
            repo_permission: None,
            checked_at,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn ready(&self) -> bool {
        matches!(self.status, McpProviderStatusCode::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRegistryEntry {
    pub provider: String,
    pub kind: McpProviderKind,
    pub status: McpProviderStatusCode,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRegistry {
    pub version: String,
    pub updated_at: u64,
    pub providers: Vec<McpRegistryEntry>,
}

impl McpRegistry {
    pub fn new(updated_at: u64) -> Self {
        Self {
            version: MCP_REGISTRY_VERSION.to_string(),
            updated_at,
            providers: Vec::new(),
        }
    }
}
