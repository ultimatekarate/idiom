use crate::pattern::{ConventionSummary, Deviation, NamingPattern, PatternKind};

/// Render a convention summary as human-readable text for `idiom infer`.
pub fn render_summary(summary: &ConventionSummary) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Conventions inferred for: {}\n\n",
        summary.directory.display()
    ));

    if summary.patterns.is_empty() {
        out.push_str("  (no patterns detected with sufficient confidence)\n");
        return out;
    }

    for pattern in &summary.patterns {
        out.push_str(&format!(
            "  {} {}: {} ({:?} confidence, {}/{} match)\n",
            pattern.role,
            pattern_kind_label(&pattern.kind),
            pattern.kind,
            pattern.confidence,
            pattern.evidence.len(),
            pattern.evidence.len() + pattern.exceptions.len(),
        ));
    }

    out
}

/// Render a convention summary as structured context for agent injection.
pub fn render_context(summary: &ConventionSummary) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "# Local conventions for {}\n\n",
        summary.directory.display()
    ));

    if summary.patterns.is_empty() {
        out.push_str("No strong conventions detected.\n");
        return out;
    }

    out.push_str("When writing code in this directory, follow these naming conventions:\n\n");

    for pattern in &summary.patterns {
        let instruction = match &pattern.kind {
            PatternKind::Prefix(p) => format!(
                "- {} names should start with '{p}' (e.g., {})",
                pattern.role,
                pattern.evidence.first().map_or("", |s| s.as_str()),
            ),
            PatternKind::Suffix(s) => format!(
                "- {} names should end with '{s}' (e.g., {})",
                pattern.role,
                pattern.evidence.first().map_or("", |s| s.as_str()),
            ),
            PatternKind::Casing(style) => format!(
                "- {} names should use {style}",
                pattern.role,
            ),
        };
        out.push_str(&instruction);
        out.push('\n');
    }

    out
}

/// Render deviations in compiler-like format for `idiom check`.
pub fn render_deviations(deviations: &[Deviation]) -> String {
    let mut out = String::new();

    for dev in deviations {
        out.push_str(&format!(
            "warning[{}]: {}\n",
            dev.code(),
            dev.message,
        ));
        out.push_str(&format!(
            "  --> {}:{}\n",
            dev.file.display(),
            dev.line,
        ));
        out.push_str(&format!(
            "  = note: local convention is {}\n",
            dev.expected_pattern,
        ));
        out.push_str(&render_help(dev));
        out.push('\n');
    }

    out
}

fn render_help(dev: &Deviation) -> String {
    match &dev.expected_pattern {
        PatternKind::Prefix(p) => format!(
            "  = help: rename to '{p}{}'\n",
            strip_existing_prefix(&dev.name),
        ),
        PatternKind::Suffix(s) => format!(
            "  = help: rename to '{}{}'\n",
            strip_existing_suffix(&dev.name),
            s,
        ),
        PatternKind::Casing(style) => format!(
            "  = help: rename to {style}\n",
        ),
    }
}

fn pattern_kind_label(kind: &PatternKind) -> &'static str {
    match kind {
        PatternKind::Prefix(_) => "prefix",
        PatternKind::Suffix(_) => "suffix",
        PatternKind::Casing(_) => "casing",
    }
}

fn strip_existing_prefix(name: &str) -> &str {
    // Strip up to and including the first underscore
    name.find('_').map_or(name, |i| &name[i + 1..])
}

fn strip_existing_suffix(name: &str) -> &str {
    // For snake_case: strip from the last underscore
    // For PascalCase: return as-is (suffix is appended)
    if name.contains('_') {
        name.rfind('_').map_or(name, |i| &name[..i])
    } else {
        name
    }
}

/// Render a convention summary as JSON for agent context injection.
pub fn render_json(patterns: &[NamingPattern]) -> String {
    serde_json::to_string_pretty(patterns).unwrap_or_default()
}

/// Render deviations as JSON for CI integration.
pub fn render_deviations_json(deviations: &[Deviation]) -> String {
    use serde_json::json;

    let entries: Vec<serde_json::Value> = deviations
        .iter()
        .map(|dev| {
            json!({
                "code": dev.code(),
                "file": dev.file.display().to_string(),
                "line": dev.line,
                "name": dev.name,
                "role": format!("{}", dev.role),
                "expected": format!("{}", dev.expected_pattern),
                "message": dev.message,
            })
        })
        .collect();

    let result = json!({
        "deviations": entries,
        "total": deviations.len(),
    });

    serde_json::to_string_pretty(&result).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{Confidence, SyntacticRole};
    use std::path::PathBuf;

    #[test]
    fn render_deviation_format() {
        let dev = Deviation {
            file: PathBuf::from("src/gates/trust.rs"),
            line: 42,
            name: "process_envelope".into(),
            role: SyntacticRole::Function,
            expected_pattern: PatternKind::Prefix("handle_".into()),
            confidence: Confidence::High,
            message: "function name 'process_envelope' does not match local convention: prefix 'handle_'"
                .into(),
        };
        let output = render_deviations(&[dev]);
        assert!(output.contains("warning[I001]"));
        assert!(output.contains("src/gates/trust.rs:42"));
        assert!(output.contains("handle_"));
    }

    #[test]
    fn render_empty_summary() {
        let summary = ConventionSummary {
            directory: PathBuf::from("src/test"),
            patterns: vec![],
        };
        let output = render_summary(&summary);
        assert!(output.contains("no patterns detected"));
    }
}
