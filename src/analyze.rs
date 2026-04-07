use crate::pattern::{
    CasingStyle, Confidence, Deviation, NamingPattern, PatternKind, SyntacticRole,
};
use crate::spec::{NamingOverrides, RoleOverride};
use std::collections::HashMap;
use std::path::Path;

/// A named declaration extracted from source code.
#[derive(Debug, Clone)]
pub struct NamedDecl {
    pub name: String,
    pub role: SyntacticRole,
    pub line: usize,
}

/// Extract naming patterns from a list of declarations.
///
/// Pure function: no IO. Takes names in, produces patterns out.
pub fn extract_patterns(decls: &[NamedDecl]) -> Vec<NamingPattern> {
    let mut patterns = Vec::new();

    let mut by_role: HashMap<SyntacticRole, Vec<&str>> = HashMap::new();
    for decl in decls {
        by_role
            .entry(decl.role)
            .or_default()
            .push(&decl.name);
    }

    for (role, names) in &by_role {
        if names.len() < 2 {
            continue;
        }

        // Detect prefix patterns
        if let Some(pattern) = detect_prefix(names, *role) {
            patterns.push(pattern);
        }

        // Detect suffix patterns
        if let Some(pattern) = detect_suffix(names, *role) {
            patterns.push(pattern);
        }

        // Detect casing patterns
        if let Some(pattern) = detect_casing(names, *role) {
            patterns.push(pattern);
        }
    }

    patterns
}

/// Check declarations against established patterns, producing deviations.
///
/// Pure function: takes patterns and declarations, returns deviations.
pub fn check_against_patterns(
    patterns: &[NamingPattern],
    decls: &[NamedDecl],
    file: &Path,
    overrides: Option<&NamingOverrides>,
) -> Vec<Deviation> {
    let mut deviations = Vec::new();

    for decl in decls {
        for pattern in patterns {
            if pattern.role != decl.role {
                continue;
            }

            // Check if this pattern kind is suppressed
            if let Some(ovr) = overrides {
                let role_override = role_override_for(ovr, decl.role);
                if is_suppressed(role_override, &pattern.kind) {
                    continue;
                }
            }

            // Only enforce High confidence patterns (or any pinned pattern)
            let pinned = is_pinned(overrides, decl.role, &pattern.kind);
            if !pinned && pattern.confidence != Confidence::High {
                continue;
            }

            if !matches_pattern(&decl.name, &pattern.kind) {
                deviations.push(Deviation {
                    file: file.to_path_buf(),
                    line: decl.line,
                    name: decl.name.clone(),
                    role: decl.role,
                    expected_pattern: pattern.kind.clone(),
                    confidence: pattern.confidence,
                    message: format!(
                        "{} name '{}' does not match local convention: {}",
                        decl.role, decl.name, pattern.kind
                    ),
                });
            }
        }
    }

    deviations
}

/// Check if a name matches a pattern kind.
pub fn matches_pattern(name: &str, kind: &PatternKind) -> bool {
    match kind {
        PatternKind::Prefix(p) => {
            if p.ends_with('_') {
                // Snake-case prefix: literal match
                name.starts_with(p.as_str())
            } else {
                // CamelCase prefix: first word must match (case-insensitive)
                use crate::language::split_identifier_words;
                let words = split_identifier_words(name);
                words.first().map_or(false, |w| w == p)
            }
        }
        PatternKind::Suffix(s) => name.ends_with(s.as_str()),
        PatternKind::Casing(style) => detect_casing_style(name) == Some(*style),
    }
}

fn role_override_for(overrides: &NamingOverrides, role: SyntacticRole) -> &RoleOverride {
    match role {
        SyntacticRole::Function => &overrides.functions,
        SyntacticRole::Type => &overrides.types,
        SyntacticRole::Module => &overrides.modules,
        SyntacticRole::Constructor => &overrides.constructors,
    }
}

fn is_suppressed(role_override: &RoleOverride, kind: &PatternKind) -> bool {
    let key = pattern_kind_key(kind);
    role_override.suppress.get(key).copied().unwrap_or(false)
}

fn is_pinned(overrides: Option<&NamingOverrides>, role: SyntacticRole, kind: &PatternKind) -> bool {
    let Some(ovr) = overrides else {
        return false;
    };
    let role_override = role_override_for(ovr, role);
    let key = pattern_kind_key(kind);
    role_override.pin.contains_key(key)
}

fn pattern_kind_key(kind: &PatternKind) -> &str {
    match kind {
        PatternKind::Prefix(_) => "prefix",
        PatternKind::Suffix(_) => "suffix",
        PatternKind::Casing(_) => "casing",
    }
}

/// Detect a common prefix among names for a given role.
/// Handles both snake_case (`handle_event`) and camelCase (`handleEvent`) prefixes.
fn detect_prefix(names: &[&str], role: SyntacticRole) -> Option<NamingPattern> {
    // Try snake_case prefix detection first
    if let Some(pattern) = detect_snake_prefix(names, role) {
        return Some(pattern);
    }

    // Try camelCase prefix detection
    detect_camel_prefix(names, role)
}

/// Detect common prefix in snake_case names (e.g., "handle_" from handle_event, handle_message).
fn detect_snake_prefix(names: &[&str], role: SyntacticRole) -> Option<NamingPattern> {
    let split_names: Vec<Vec<&str>> = names
        .iter()
        .map(|n| n.split('_').collect::<Vec<_>>())
        .collect();

    let multi_segment: Vec<&Vec<&str>> = split_names.iter().filter(|s| s.len() >= 2).collect();
    if multi_segment.len() < 2 {
        return None;
    }

    // Count frequency of each first segment
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for segments in &multi_segment {
        *freq.entry(segments[0]).or_default() += 1;
    }

    let (best_segment, count) = freq.into_iter().max_by_key(|(_, c)| *c)?;
    if count < 2 || best_segment.is_empty() {
        return None;
    }

    let mut matching = Vec::new();
    let mut exceptions = Vec::new();

    for (i, segments) in split_names.iter().enumerate() {
        if segments.len() >= 2 && segments[0] == best_segment {
            matching.push(names[i].to_string());
        } else {
            exceptions.push(names[i].to_string());
        }
    }

    let ratio = matching.len() as f64 / names.len() as f64;
    let confidence = ratio_to_confidence(ratio);

    let prefix = format!("{best_segment}_");

    Some(NamingPattern {
        role,
        kind: PatternKind::Prefix(prefix),
        confidence,
        evidence: matching,
        exceptions,
    })
}

/// Detect common prefix in camelCase names (e.g., "handle" from handleEvent, handleMessage).
fn detect_camel_prefix(names: &[&str], role: SyntacticRole) -> Option<NamingPattern> {
    use crate::language::split_identifier_words;

    let word_lists: Vec<Vec<String>> = names
        .iter()
        .map(|n| split_identifier_words(n))
        .collect();

    // Only consider names with 2+ words
    let multi_word: Vec<(usize, &Vec<String>)> = word_lists
        .iter()
        .enumerate()
        .filter(|(_, w)| w.len() >= 2)
        .collect();

    if multi_word.len() < 2 {
        return None;
    }

    // Count frequency of each first word
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for (_, words) in &multi_word {
        *freq.entry(&words[0]).or_default() += 1;
    }

    let (best_word, count) = freq.into_iter().max_by_key(|(_, c)| *c)?;
    if count < 2 || best_word.is_empty() {
        return None;
    }

    let mut matching = Vec::new();
    let mut exceptions = Vec::new();

    for (i, words) in word_lists.iter().enumerate() {
        if words.len() >= 2 && words[0] == best_word {
            matching.push(names[i].to_string());
        } else {
            exceptions.push(names[i].to_string());
        }
    }

    let ratio = matching.len() as f64 / names.len() as f64;
    let confidence = ratio_to_confidence(ratio);

    // Reconstruct the prefix in the original casing style
    let prefix = best_word.to_string();

    Some(NamingPattern {
        role,
        kind: PatternKind::Prefix(prefix),
        confidence,
        evidence: matching,
        exceptions,
    })
}

/// Detect a common suffix among names for a given role.
fn detect_suffix(names: &[&str], role: SyntacticRole) -> Option<NamingPattern> {
    // For PascalCase types, look for common trailing words
    // For snake_case, look for common trailing underscore segments

    // Try PascalCase suffix detection (e.g., "Gate", "Actor", "Error")
    let pascal_suffixes: Vec<Option<String>> = names
        .iter()
        .map(|n| extract_pascal_suffix(n))
        .collect();

    let valid_suffixes: Vec<&str> = pascal_suffixes
        .iter()
        .filter_map(|s| s.as_deref())
        .collect();

    if valid_suffixes.len() >= 2 {
        // Count frequency of each suffix
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for s in &valid_suffixes {
            *freq.entry(s).or_default() += 1;
        }

        if let Some((best_suffix, count)) = freq.iter().max_by_key(|(_, c)| **c) {
            if *count >= 2 {
                let mut matching = Vec::new();
                let mut exceptions = Vec::new();

                for name in names {
                    if extract_pascal_suffix(name).as_deref() == Some(*best_suffix) {
                        matching.push(name.to_string());
                    } else {
                        exceptions.push(name.to_string());
                    }
                }

                let ratio = matching.len() as f64 / names.len() as f64;
                let confidence = ratio_to_confidence(ratio);

                return Some(NamingPattern {
                    role,
                    kind: PatternKind::Suffix(best_suffix.to_string()),
                    confidence,
                    evidence: matching,
                    exceptions,
                });
            }
        }
    }

    // Try snake_case suffix detection (e.g., "_id", "_name")
    let split_names: Vec<Vec<&str>> = names
        .iter()
        .map(|n| n.split('_').collect::<Vec<_>>())
        .collect();

    let multi_segment: Vec<&Vec<&str>> = split_names.iter().filter(|s| s.len() >= 2).collect();
    if multi_segment.len() < 2 {
        return None;
    }

    // Count frequency of each last segment
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for segments in &multi_segment {
        if let Some(last) = segments.last() {
            if !last.is_empty() {
                *freq.entry(last).or_default() += 1;
            }
        }
    }

    let (best_segment, count) = freq.into_iter().max_by_key(|(_, c)| *c)?;
    if count < 2 {
        return None;
    }

    let mut matching = Vec::new();
    let mut exceptions = Vec::new();

    for (i, segments) in split_names.iter().enumerate() {
        if segments.len() >= 2 && segments.last() == Some(&best_segment) {
            matching.push(names[i].to_string());
        } else {
            exceptions.push(names[i].to_string());
        }
    }

    let ratio = matching.len() as f64 / names.len() as f64;
    let confidence = ratio_to_confidence(ratio);

    let suffix = format!("_{best_segment}");

    Some(NamingPattern {
        role,
        kind: PatternKind::Suffix(suffix),
        confidence,
        evidence: matching,
        exceptions,
    })
}

/// Extract the last PascalCase word from an identifier.
/// "TrustGate" -> "Gate", "MeshSentinel" -> "Sentinel", "foo" -> None
fn extract_pascal_suffix(name: &str) -> Option<String> {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < 2 {
        return None;
    }

    // Walk backwards to find the last uppercase letter that starts a word
    let mut last_upper_start = None;
    for i in (1..chars.len()).rev() {
        if chars[i].is_uppercase() {
            last_upper_start = Some(i);
            break;
        }
    }

    let start = last_upper_start?;
    let suffix: String = chars[start..].iter().collect();

    // Don't return the whole name as a suffix
    if start == 0 {
        return None;
    }

    // Must be at least 2 chars to be meaningful
    if suffix.len() < 2 {
        return None;
    }

    Some(suffix)
}

/// Detect the predominant casing style among names.
fn detect_casing(names: &[&str], role: SyntacticRole) -> Option<NamingPattern> {
    let mut counts: HashMap<CasingStyle, Vec<String>> = HashMap::new();

    for name in names {
        if let Some(style) = detect_casing_style(name) {
            counts.entry(style).or_default().push(name.to_string());
        }
    }

    let (best_style, matching) = counts.into_iter().max_by_key(|(_, v)| v.len())?;

    if matching.len() < 2 {
        return None;
    }

    let exceptions: Vec<String> = names
        .iter()
        .filter(|n| detect_casing_style(n) != Some(best_style))
        .map(|n| n.to_string())
        .collect();

    let ratio = matching.len() as f64 / names.len() as f64;
    let confidence = ratio_to_confidence(ratio);

    Some(NamingPattern {
        role,
        kind: PatternKind::Casing(best_style),
        confidence,
        evidence: matching,
        exceptions,
    })
}

/// Detect the casing style of a single identifier.
pub fn detect_casing_style(name: &str) -> Option<CasingStyle> {
    if name.is_empty() {
        return None;
    }

    let has_underscore = name.contains('_');
    let has_uppercase = name.chars().any(|c| c.is_uppercase());
    let has_lowercase = name.chars().any(|c| c.is_lowercase());
    let starts_upper = name.chars().next().map_or(false, |c| c.is_uppercase());

    if has_underscore && !has_lowercase && has_uppercase {
        Some(CasingStyle::ScreamingSnakeCase)
    } else if has_underscore && has_lowercase {
        Some(CasingStyle::SnakeCase)
    } else if starts_upper && has_lowercase && !has_underscore {
        Some(CasingStyle::PascalCase)
    } else if !starts_upper && has_uppercase && has_lowercase && !has_underscore {
        Some(CasingStyle::CamelCase)
    } else if has_lowercase && !has_uppercase && !has_underscore {
        // All lowercase, single word — could be snake_case (just one segment)
        Some(CasingStyle::SnakeCase)
    } else {
        None
    }
}

fn ratio_to_confidence(ratio: f64) -> Confidence {
    // Use direct comparison since these are exact fractions
    if ratio >= 1.0 - f64::EPSILON {
        Confidence::High
    } else if ratio >= 0.8 - f64::EPSILON {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_snake_case_style() {
        assert_eq!(detect_casing_style("handle_event"), Some(CasingStyle::SnakeCase));
        assert_eq!(detect_casing_style("my_var"), Some(CasingStyle::SnakeCase));
    }

    #[test]
    fn detect_camel_case_style() {
        assert_eq!(detect_casing_style("handleEvent"), Some(CasingStyle::CamelCase));
        assert_eq!(detect_casing_style("myVar"), Some(CasingStyle::CamelCase));
    }

    #[test]
    fn detect_pascal_case_style() {
        assert_eq!(detect_casing_style("HandleEvent"), Some(CasingStyle::PascalCase));
        assert_eq!(detect_casing_style("TrustGate"), Some(CasingStyle::PascalCase));
    }

    #[test]
    fn detect_screaming_snake() {
        assert_eq!(detect_casing_style("MAX_SIZE"), Some(CasingStyle::ScreamingSnakeCase));
    }

    #[test]
    fn extract_prefix_from_functions() {
        let decls = vec![
            NamedDecl { name: "handle_event".into(), role: SyntacticRole::Function, line: 1 },
            NamedDecl { name: "handle_message".into(), role: SyntacticRole::Function, line: 5 },
            NamedDecl { name: "handle_timeout".into(), role: SyntacticRole::Function, line: 9 },
        ];
        let patterns = extract_patterns(&decls);
        let prefix_pat = patterns.iter().find(|p| matches!(p.kind, PatternKind::Prefix(_)));
        assert!(prefix_pat.is_some());
        let pat = prefix_pat.unwrap();
        assert_eq!(pat.kind, PatternKind::Prefix("handle_".into()));
        assert_eq!(pat.confidence, Confidence::High);
    }

    #[test]
    fn extract_suffix_from_types() {
        let decls = vec![
            NamedDecl { name: "TrustGate".into(), role: SyntacticRole::Type, line: 1 },
            NamedDecl { name: "IntegrityGate".into(), role: SyntacticRole::Type, line: 10 },
            NamedDecl { name: "TopologyGate".into(), role: SyntacticRole::Type, line: 20 },
        ];
        let patterns = extract_patterns(&decls);
        let suffix_pat = patterns.iter().find(|p| matches!(p.kind, PatternKind::Suffix(_)));
        assert!(suffix_pat.is_some());
        let pat = suffix_pat.unwrap();
        assert_eq!(pat.kind, PatternKind::Suffix("Gate".into()));
        assert_eq!(pat.confidence, Confidence::High);
    }

    #[test]
    fn check_deviation_detected() {
        let patterns = vec![NamingPattern {
            role: SyntacticRole::Function,
            kind: PatternKind::Prefix("handle_".into()),
            confidence: Confidence::High,
            evidence: vec!["handle_event".into(), "handle_message".into()],
            exceptions: vec![],
        }];
        let decls = vec![NamedDecl {
            name: "process_data".into(),
            role: SyntacticRole::Function,
            line: 42,
        }];
        let devs = check_against_patterns(&patterns, &decls, Path::new("test.rs"), None);
        assert_eq!(devs.len(), 1);
        assert_eq!(devs[0].name, "process_data");
        assert_eq!(devs[0].code(), "I001");
    }

    #[test]
    fn matching_name_produces_no_deviation() {
        let patterns = vec![NamingPattern {
            role: SyntacticRole::Function,
            kind: PatternKind::Prefix("handle_".into()),
            confidence: Confidence::High,
            evidence: vec!["handle_event".into()],
            exceptions: vec![],
        }];
        let decls = vec![NamedDecl {
            name: "handle_request".into(),
            role: SyntacticRole::Function,
            line: 10,
        }];
        let devs = check_against_patterns(&patterns, &decls, Path::new("test.rs"), None);
        assert!(devs.is_empty());
    }

    #[test]
    fn medium_confidence_not_enforced_by_default() {
        let patterns = vec![NamingPattern {
            role: SyntacticRole::Function,
            kind: PatternKind::Prefix("handle_".into()),
            confidence: Confidence::Medium,
            evidence: vec!["handle_event".into()],
            exceptions: vec!["other".into()],
        }];
        let decls = vec![NamedDecl {
            name: "process_data".into(),
            role: SyntacticRole::Function,
            line: 1,
        }];
        let devs = check_against_patterns(&patterns, &decls, Path::new("test.rs"), None);
        assert!(devs.is_empty());
    }

    #[test]
    fn pascal_suffix_extraction() {
        assert_eq!(extract_pascal_suffix("TrustGate"), Some("Gate".into()));
        assert_eq!(extract_pascal_suffix("MeshSentinel"), Some("Sentinel".into()));
        assert_eq!(extract_pascal_suffix("foo"), None);
        assert_eq!(extract_pascal_suffix("A"), None);
    }

    #[test]
    fn extract_camel_case_prefix() {
        let decls = vec![
            NamedDecl { name: "handleEvent".into(), role: SyntacticRole::Function, line: 1 },
            NamedDecl { name: "handleMessage".into(), role: SyntacticRole::Function, line: 5 },
            NamedDecl { name: "handleTimeout".into(), role: SyntacticRole::Function, line: 9 },
        ];
        let patterns = extract_patterns(&decls);
        let prefix_pat = patterns.iter().find(|p| matches!(p.kind, PatternKind::Prefix(_)));
        assert!(prefix_pat.is_some());
        let pat = prefix_pat.unwrap();
        assert_eq!(pat.kind, PatternKind::Prefix("handle".into()));
        assert_eq!(pat.confidence, Confidence::High);
    }

    #[test]
    fn camel_case_prefix_deviation() {
        let patterns = vec![NamingPattern {
            role: SyntacticRole::Function,
            kind: PatternKind::Prefix("handle".into()),
            confidence: Confidence::High,
            evidence: vec!["handleEvent".into(), "handleMessage".into()],
            exceptions: vec![],
        }];
        let decls = vec![NamedDecl {
            name: "processData".into(),
            role: SyntacticRole::Function,
            line: 42,
        }];
        let devs = check_against_patterns(&patterns, &decls, Path::new("test.js"), None);
        assert_eq!(devs.len(), 1);
        assert_eq!(devs[0].name, "processData");
    }

    #[test]
    fn snake_suffix_picks_most_common() {
        // The first name ends in "_count" but the majority end in "_id".
        // Old code would pick "_count" (first name's suffix); fixed code picks "_id".
        let decls = vec![
            NamedDecl { name: "session_count".into(), role: SyntacticRole::Function, line: 1 },
            NamedDecl { name: "user_id".into(), role: SyntacticRole::Function, line: 2 },
            NamedDecl { name: "order_id".into(), role: SyntacticRole::Function, line: 3 },
            NamedDecl { name: "product_id".into(), role: SyntacticRole::Function, line: 4 },
        ];
        let patterns = extract_patterns(&decls);
        let suffix_pat = patterns.iter().find(|p| matches!(p.kind, PatternKind::Suffix(_)));
        assert!(suffix_pat.is_some(), "Expected a suffix pattern");
        let pat = suffix_pat.unwrap();
        assert_eq!(pat.kind, PatternKind::Suffix("_id".into()));
        assert_eq!(pat.evidence.len(), 3);
        assert_eq!(pat.exceptions.len(), 1);
    }

    #[test]
    fn camel_case_prefix_match_passes() {
        let patterns = vec![NamingPattern {
            role: SyntacticRole::Function,
            kind: PatternKind::Prefix("handle".into()),
            confidence: Confidence::High,
            evidence: vec!["handleEvent".into()],
            exceptions: vec![],
        }];
        let decls = vec![NamedDecl {
            name: "handleRequest".into(),
            role: SyntacticRole::Function,
            line: 10,
        }];
        let devs = check_against_patterns(&patterns, &decls, Path::new("test.js"), None);
        assert!(devs.is_empty());
    }
}
