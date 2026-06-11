use crate::model::{McpProviderStatus, McpRegistry, McpRegistryEntry};

pub fn build_registry(statuses: &[McpProviderStatus], updated_at: u64) -> McpRegistry {
    let mut registry = McpRegistry::new(updated_at);
    registry.providers = statuses
        .iter()
        .map(|status| McpRegistryEntry {
            provider: status.provider.clone(),
            kind: status.kind.clone(),
            status: status.status.clone(),
            path: format!(".agentflow/state/mcp/providers/{}.json", status.provider),
        })
        .collect();
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{McpProviderKind, McpProviderStatus, McpProviderStatusCode};

    #[test]
    fn registry_points_to_provider_files() {
        let mut github = McpProviderStatus::new(McpProviderKind::Github, 1);
        github.status = McpProviderStatusCode::Ready;
        let registry = build_registry(&[github], 2);

        assert_eq!(registry.providers.len(), 1);
        assert_eq!(
            registry.providers[0].path,
            ".agentflow/state/mcp/providers/github.json"
        );
    }
}
