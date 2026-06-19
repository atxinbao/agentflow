use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::model::OntologyBundle;

pub fn read_ontology_bundle(path: impl AsRef<Path>) -> Result<OntologyBundle> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn write_ontology_bundle(path: impl AsRef<Path>, bundle: &OntologyBundle) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(bundle)?;
    fs::write(path, content)?;
    Ok(())
}
