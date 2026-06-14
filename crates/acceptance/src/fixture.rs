use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

pub struct WorkflowFixture {
    temp_dir: TempDir,
    baseline_hashes: BTreeMap<String, String>,
}

impl WorkflowFixture {
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn assert_user_files_unchanged(&self) -> Result<()> {
        for (relative_path, expected_hash) in &self.baseline_hashes {
            let actual_hash = hash_file(&self.root().join(relative_path))?;
            anyhow::ensure!(
                &actual_hash == expected_hash,
                "fixture user file changed: {relative_path}"
            );
        }
        Ok(())
    }
}

pub fn create_fixture_project() -> Result<WorkflowFixture> {
    let temp_dir = tempfile::tempdir().context("create workflow acceptance fixture")?;
    let root = temp_dir.path();
    fs::create_dir_all(root.join("src")).context("create fixture src directory")?;
    fs::write(root.join("README.md"), "# AgentFlow acceptance fixture\n")?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"agentflow-acceptance-fixture\"\nedition = \"2021\"\nversion = \"0.0.0\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )?;
    fs::write(
        root.join("src/lib.rs"),
        "pub fn value() -> u8 {\n    1\n}\n",
    )?;

    let baseline_hashes = ["README.md", "Cargo.toml", "src/lib.rs"]
        .into_iter()
        .map(|relative_path| {
            Ok((
                relative_path.to_string(),
                hash_file(&root.join(relative_path))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    Ok(WorkflowFixture {
        temp_dir,
        baseline_hashes,
    })
}

fn hash_file(path: &PathBuf) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
