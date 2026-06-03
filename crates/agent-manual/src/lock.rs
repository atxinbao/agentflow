use crate::{
    hash::sha256_hex,
    model::{
        SkillsLock, SkillsLockEntry, SkillsLockItem, AGENT_ENTRY_VERSION, AGENT_MANUAL_VERSION,
        LOCK_VERSION,
    },
    templates::{
        agent_md_template, agentflow_manual_template, skill_templates, skill_version,
        AGENT_MANUAL_RELATIVE_PATH,
    },
};
use anyhow::{Context, Result};
use std::{collections::BTreeMap, fs, path::Path};

pub fn expected_skills_lock(updated_at: u64) -> SkillsLock {
    let skills = skill_templates()
        .into_iter()
        .map(|skill| {
            (
                skill.name.to_string(),
                SkillsLockItem {
                    version: skill_version().to_string(),
                    path: skill.relative_path.to_string(),
                    hash: sha256_hex(skill.content),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    SkillsLock {
        version: LOCK_VERSION.to_string(),
        managed_by: "AgentFlow".to_string(),
        updated_at,
        entry: SkillsLockEntry {
            path: "AGENT.MD".to_string(),
            version: AGENT_ENTRY_VERSION.to_string(),
            managed: true,
            hash: sha256_hex(&agent_md_template()),
        },
        manual: SkillsLockItem {
            version: AGENT_MANUAL_VERSION.to_string(),
            path: AGENT_MANUAL_RELATIVE_PATH.to_string(),
            hash: sha256_hex(&agentflow_manual_template()),
        },
        skills,
    }
}

pub fn read_skills_lock(path: &Path) -> Result<SkillsLock> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
