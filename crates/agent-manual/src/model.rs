use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STATUS_VERSION: &str = "agent-environment-status.v1";
pub const LOCK_VERSION: &str = "agentflow-skills-lock.v1";
pub const AGENT_ENTRY_VERSION: &str = "agent-entry.v2";
pub const AGENT_MANUAL_VERSION: &str = "agentflow-manual.v1";
pub const SKILL_VERSION: &str = "v1";
pub const WORKSPACE_MANIFEST_VERSION: &str = "agentflow-workspace-manifest.v1";
pub const WORKSPACE_LAYOUT_VERSION: &str = "agentflow-layout.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEnvironmentState {
    Missing,
    Checking,
    Repairing,
    Ready,
    Repaired,
    Degraded,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEnvironmentStatus {
    pub version: String,
    pub project_root: String,
    pub status: AgentEnvironmentState,
    pub ready: bool,
    pub checked_at: u64,
    pub repaired_at: Option<u64>,
    pub agent_md: AgentMdStatus,
    pub manual: ManualStatus,
    pub skills_lock: SkillsLockStatus,
    pub skills: Vec<SkillStatus>,
    pub repairs: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub workspace_manifest: WorkspaceManifestStatus,
    pub layout: WorkspaceLayoutStatus,
    pub legacy_agent_entry: LegacyAgentEntryStatus,
    pub shadow_guard: RootAgentEntryShadowGuardStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMdStatus {
    pub exists: bool,
    pub managed: bool,
    pub version: Option<String>,
    pub hash: Option<String>,
    pub backed_up: bool,
    pub tracked_by_git: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyAgentEntryStatus {
    pub exists: bool,
    pub path: String,
    pub managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualStatus {
    pub exists: bool,
    pub path: String,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsLockStatus {
    pub exists: bool,
    pub valid: bool,
    pub path: String,
    pub skill_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStatus {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub hash_matches: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceManifestStatus {
    pub exists: bool,
    pub path: String,
    pub valid: bool,
    pub layout_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLayoutStatus {
    pub version: String,
    pub ready: bool,
    pub created_paths: Vec<String>,
    pub reused_paths: Vec<String>,
    pub missing_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootAgentEntryShadowGuardStatus {
    pub checked: Vec<String>,
    pub detected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsLock {
    pub version: String,
    pub managed_by: String,
    pub updated_at: u64,
    pub entry: SkillsLockEntry,
    pub manual: SkillsLockItem,
    pub skills: BTreeMap<String, SkillsLockItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsLockEntry {
    pub path: String,
    pub version: String,
    pub managed: bool,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceManifest {
    pub version: String,
    pub layout_version: String,
    pub project_root: String,
    pub root_entries: WorkspaceManifestRootEntries,
    pub active_layers: Vec<String>,
    pub planned_layers: Vec<String>,
    pub paths: BTreeMap<String, String>,
    pub compat: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceManifestRootEntries {
    pub canonical_agent_entry: String,
    pub legacy_agent_entry: String,
    pub shadow_checked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsLockItem {
    pub version: String,
    pub path: String,
    pub hash: String,
}
