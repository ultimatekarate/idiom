use crate::spec::{OverrideSpec, SpecError};
use std::path::Path;

const OVERRIDE_FILENAME: &str = ".idiom.yaml";

/// Load an override spec from a directory, if one exists.
/// Returns Ok(None) if no override file is present.
pub fn load_overrides(dir: &Path) -> Result<Option<OverrideSpec>, SpecError> {
    let path = dir.join(OVERRIDE_FILENAME);
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    let spec: OverrideSpec = serde_yaml::from_str(&content)?;
    Ok(Some(spec))
}
