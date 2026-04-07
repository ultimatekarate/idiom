use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The override spec loaded from `.idiom.yaml` in a directory.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OverrideSpec {
    #[serde(default)]
    pub naming: NamingOverrides,
}

/// Naming overrides organized by syntactic role.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NamingOverrides {
    #[serde(default)]
    pub functions: RoleOverride,
    #[serde(default)]
    pub types: RoleOverride,
    #[serde(default)]
    pub modules: RoleOverride,
    #[serde(default)]
    pub constructors: RoleOverride,
}

/// Pin or suppress patterns for a single syntactic role.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RoleOverride {
    #[serde(default)]
    pub pin: HashMap<String, String>,
    #[serde(default)]
    pub suppress: HashMap<String, bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("Failed to read override file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse override YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_empty_override() {
        let yaml = "naming: {}\n";
        let spec: OverrideSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(spec.naming.functions.pin.is_empty());
    }

    #[test]
    fn deserialize_pin_and_suppress() {
        let yaml = r#"
naming:
  functions:
    pin:
      prefix: "handle_"
    suppress:
      suffix: true
  types:
    pin:
      suffix: "Gate"
"#;
        let spec: OverrideSpec = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(spec.naming.functions.pin.get("prefix").unwrap(), "handle_");
        assert!(spec.naming.functions.suppress.get("suffix").copied().unwrap_or(false));
        assert_eq!(spec.naming.types.pin.get("suffix").unwrap(), "Gate");
    }
}
