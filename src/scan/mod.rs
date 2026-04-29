use crate::analyze::{self, NamedDecl};
use crate::language::LangRegistry;
use crate::loader;
use crate::pattern::{ConventionSummary, Deviation};
use std::path::Path;

/// Extract all named declarations from source files in a directory.
/// Only scans files at the same directory level (siblings), not recursively.
/// If `exclude` is Some, that file is skipped (used by check to avoid self-bias).
pub fn extract_sibling_decls(
    dir: &Path,
    registry: &LangRegistry,
    exclude: Option<&Path>,
) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return decls,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip the excluded file (typically the target being checked)
        if let Some(excl) = exclude {
            if same_file(&path, excl) {
                continue;
            }
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let Some(lang) = registry.for_ext(ext) else {
            continue;
        };

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_decls = (lang.extract_names)(&content);
        decls.extend(file_decls);
    }

    decls
}

/// Compare two paths for equality, normalizing to canonical form.
fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

/// Extract declarations from a single file.
pub fn extract_file_decls(file: &Path, registry: &LangRegistry) -> Vec<NamedDecl> {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let Some(lang) = registry.for_ext(ext) else {
        return Vec::new();
    };

    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    (lang.extract_names)(&content)
}

/// Infer conventions for a directory: extract sibling names, analyze patterns.
pub fn infer_conventions(dir: &Path, registry: &LangRegistry) -> ConventionSummary {
    let decls = extract_sibling_decls(dir, registry, None);
    let patterns = analyze::extract_patterns(&decls);

    ConventionSummary {
        directory: dir.to_path_buf(),
        patterns,
    }
}

/// Check a file against the local conventions of its directory.
pub fn check_file(file: &Path, registry: &LangRegistry) -> Vec<Deviation> {
    let dir = match file.parent() {
        Some(d) => d,
        None => return Vec::new(),
    };

    // Load conventions from sibling files, excluding the target to avoid self-bias
    let sibling_decls = extract_sibling_decls(dir, registry, Some(file));
    let patterns = analyze::extract_patterns(&sibling_decls);

    // Load overrides if present
    let overrides = loader::load_overrides(dir).ok().flatten();
    let naming_overrides = overrides.as_ref().map(|o| &o.naming);

    // Extract declarations from the target file
    let target_decls = extract_file_decls(file, registry);

    // Check target against sibling patterns
    analyze::check_against_patterns(&patterns, &target_decls, file, naming_overrides)
}
