use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn assert_write_boundary(root: &Path) -> Result<()> {
    let mut top_level = fs::read_dir(root)
        .with_context(|| format!("read fixture root {}", root.display()))?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    top_level.sort();

    for entry in top_level {
        anyhow::ensure!(
            matches!(
                entry.as_str(),
                ".agentflow" | "AGENTS.md" | "Cargo.toml" | "README.md" | "src"
            ),
            "unexpected top-level write in fixture: {entry}"
        );
    }
    assert_no_forbidden_side_effects(root)
}

pub fn assert_no_forbidden_side_effects(root: &Path) -> Result<()> {
    anyhow::ensure!(
        !root.join(".remote-pr").exists(),
        "fixture must not create remote PR marker"
    );
    anyhow::ensure!(
        !root.join(".agentflow/deploy").exists(),
        "fixture must not create deploy records"
    );
    anyhow::ensure!(
        !root.join(".agentflow/goal-tree").exists(),
        "fixture must not write legacy goal-tree facts"
    );
    anyhow::ensure!(
        !root.join(".git/refs/heads/agentflow-e2e").exists(),
        "fixture must not create agentflow git branch"
    );
    Ok(())
}
