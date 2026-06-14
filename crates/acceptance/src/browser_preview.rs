use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

pub struct BrowserPreviewSmokeContract {
    pub package_script: String,
    pub browser_preview_data: String,
    pub desktop_app: String,
}

pub fn load_browser_preview_smoke_contract() -> Result<BrowserPreviewSmokeContract> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .context("resolve repository root")?
        .to_path_buf();
    Ok(BrowserPreviewSmokeContract {
        package_script: fs::read_to_string(repo_root.join("apps/desktop/package.json"))
            .context("read desktop package.json")?,
        browser_preview_data: fs::read_to_string(
            repo_root.join("apps/desktop/src/browserPreviewData.ts"),
        )
        .context("read browserPreviewData.ts")?,
        desktop_app: fs::read_to_string(repo_root.join("apps/desktop/src/App.tsx"))
            .context("read App.tsx")?,
    })
}
