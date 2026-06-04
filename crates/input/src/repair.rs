use crate::{manager::prepare_input_workspace, model::InputSnapshot};
use anyhow::Result;
use std::path::Path;

pub fn repair_input_workspace(project_root: impl AsRef<Path>) -> Result<InputSnapshot> {
    prepare_input_workspace(project_root)
}
