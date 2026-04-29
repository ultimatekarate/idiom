use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Confidence in an inferred naming pattern based on consistency among siblings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// 100% of siblings match.
    High,
    /// >= 80% of siblings match.
    Medium,
    /// < 80% match. Not enforced unless pinned.
    Low,
}

/// The syntactic role of a named declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyntacticRole {
    Function,
    Type,
    Module,
    Constructor,
}

/// The kind of naming pattern detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    /// Names share a common prefix (e.g., "handle_").
    Prefix(String),
    /// Names share a common suffix (e.g., "Gate").
    Suffix(String),
    /// Names follow a casing convention.
    Casing(CasingStyle),
}

/// Casing conventions detected in identifiers.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CasingStyle {
    SnakeCase,
    CamelCase,
    PascalCase,
    ScreamingSnakeCase,
}

/// A detected naming pattern for a syntactic role in a directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingPattern {
    pub role: SyntacticRole,
    pub kind: PatternKind,
    pub confidence: Confidence,
    /// Names that match this pattern.
    pub evidence: Vec<String>,
    /// Names that do not match this pattern.
    pub exceptions: Vec<String>,
}

/// The full set of inferred conventions for a directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionSummary {
    pub directory: PathBuf,
    pub patterns: Vec<NamingPattern>,
}

/// A deviation from local convention found in a target file.
#[derive(Debug, Clone)]
pub struct Deviation {
    pub file: PathBuf,
    pub line: usize,
    pub name: String,
    pub role: SyntacticRole,
    pub expected_pattern: PatternKind,
    pub message: String,
}

impl Deviation {
    /// Error code based on syntactic role.
    pub fn code(&self) -> &'static str {
        match self.role {
            SyntacticRole::Function => "I001",
            SyntacticRole::Type => "I002",
            SyntacticRole::Module => "I003",
            SyntacticRole::Constructor => "I004",
        }
    }
}

impl std::fmt::Display for CasingStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CasingStyle::SnakeCase => write!(f, "snake_case"),
            CasingStyle::CamelCase => write!(f, "camelCase"),
            CasingStyle::PascalCase => write!(f, "PascalCase"),
            CasingStyle::ScreamingSnakeCase => write!(f, "SCREAMING_SNAKE_CASE"),
        }
    }
}

impl std::fmt::Display for PatternKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternKind::Prefix(p) => write!(f, "prefix '{p}'"),
            PatternKind::Suffix(s) => write!(f, "suffix '{s}'"),
            PatternKind::Casing(c) => write!(f, "{c}"),
        }
    }
}

impl std::fmt::Display for SyntacticRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntacticRole::Function => write!(f, "function"),
            SyntacticRole::Type => write!(f, "type"),
            SyntacticRole::Module => write!(f, "module"),
            SyntacticRole::Constructor => write!(f, "constructor"),
        }
    }
}
