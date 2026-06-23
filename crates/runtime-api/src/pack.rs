use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub type PackRegistryView = agentflow_pack::PackRegistry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackRegistryReadReceipt {
    pub writes_authority: bool,
    pub entry_count: usize,
}

pub fn get_pack_registry(project_root: impl AsRef<Path>) -> Result<PackRegistryView> {
    agentflow_pack::load_pack_registry(project_root)
}

pub fn pack_registry_read_receipt(registry: &PackRegistryView) -> PackRegistryReadReceipt {
    PackRegistryReadReceipt {
        writes_authority: registry.writes_authority,
        entry_count: registry.entries.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::{get_pack_registry, pack_registry_read_receipt};

    #[test]
    fn runtime_reads_pack_registry_without_authority_write() {
        let dir = tempfile::tempdir().unwrap();

        let registry = get_pack_registry(dir.path()).unwrap();
        let receipt = pack_registry_read_receipt(&registry);

        assert!(!receipt.writes_authority);
        assert_eq!(receipt.entry_count, 0);
    }
}
