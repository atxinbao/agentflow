use crate::{
    health::unix_timestamp_seconds,
    model::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode},
};

pub fn browser_preview_status() -> McpProviderStatus {
    let mut status =
        McpProviderStatus::new(McpProviderKind::BrowserPreview, unix_timestamp_seconds());
    status.status = McpProviderStatusCode::Ready;
    status.installed = true;
    status.capabilities = vec![
        McpCapability::new("browser_preview.smoke", true),
        McpCapability::new("browser_preview.dom_snapshot", true),
        McpCapability::new("browser_preview.console_logs", true),
        McpCapability::new("browser_preview.screenshot", true),
    ];
    status
}
