use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct BrowserPreviewSmokeContract {
    pub package_script: String,
    pub browser_preview_data: String,
    pub desktop_app: String,
}

pub fn load_browser_preview_smoke_contract() -> Result<BrowserPreviewSmokeContract> {
    let repo_root = resolve_repo_root()?;
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

fn resolve_repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(root) = find_repo_root_from(&manifest_dir) {
        return Ok(root);
    }
    let current_dir = std::env::current_dir().context("read current directory")?;
    if let Some(root) = find_repo_root_from(&current_dir) {
        return Ok(root);
    }
    anyhow::bail!(
        "resolve repository root from {} or {}",
        manifest_dir.display(),
        current_dir.display()
    )
}

fn find_repo_root_from(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join("apps/desktop/package.json").is_file())
        .map(Path::to_path_buf)
}
