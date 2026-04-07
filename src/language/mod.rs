pub mod csharp;
pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod python;
pub mod ruby;
pub mod rust_lang;
pub mod swift;

use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;
use std::collections::HashMap;

/// A language-specific name extractor.
pub struct LangDef {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub extract_names: fn(&str) -> Vec<NamedDecl>,
}

/// Registry mapping file extensions to language definitions.
pub struct LangRegistry {
    langs: Vec<&'static LangDef>,
    ext_map: HashMap<&'static str, usize>,
}

impl Default for LangRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LangRegistry {
    pub fn new() -> Self {
        let langs: Vec<&'static LangDef> = vec![
            &python::PYTHON,
            &rust_lang::RUST,
            &javascript::JAVASCRIPT,
            &go::GO,
            &java::JAVA,
            &kotlin::KOTLIN,
            &ruby::RUBY,
            &swift::SWIFT,
            &csharp::CSHARP,
        ];
        let mut ext_map = HashMap::new();
        for (i, lang) in langs.iter().enumerate() {
            for ext in lang.extensions {
                ext_map.insert(*ext, i);
            }
        }
        LangRegistry { langs, ext_map }
    }

    pub fn for_ext(&self, ext: &str) -> Option<&'static LangDef> {
        self.ext_map.get(ext).map(|&i| self.langs[i])
    }
}

/// Split an identifier into lowercase words at camelCase, underscore, or acronym boundaries.
pub fn split_identifier_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = name.chars().collect();
    for i in 0..chars.len() {
        let ch = chars[i];

        if ch == '_' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }

        if ch.is_uppercase() {
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            let prev_is_upper = i > 0 && chars[i - 1].is_uppercase();

            if !current.is_empty() && (!prev_is_upper || next_is_lower) {
                words.push(std::mem::take(&mut current));
            }
            current.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            current.push(ch.to_lowercase().next().unwrap_or(ch));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Helper: extract function and type declarations from lines using generic patterns.
/// Used by languages that follow common declaration syntaxes.
pub fn extract_with_patterns(
    content: &str,
    fn_keywords: &[&str],
    type_keywords: &[&str],
) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            continue;
        }

        // Check function declarations
        for kw in fn_keywords {
            if let Some(rest) = find_keyword_in_line(trimmed, kw) {
                let name = extract_name_before_paren(rest);
                if !name.is_empty() && !name.starts_with('_') {
                    decls.push(NamedDecl {
                        name: name.to_string(),
                        role: SyntacticRole::Function,
                        line: line_num + 1,
                    });
                }
                break;
            }
        }

        // Check type declarations
        for kw in type_keywords {
            if let Some(rest) = find_keyword_in_line(trimmed, kw) {
                let name = extract_name_before_delimiter(rest);
                if !name.is_empty() && !name.starts_with('_') {
                    decls.push(NamedDecl {
                        name: name.to_string(),
                        role: SyntacticRole::Type,
                        line: line_num + 1,
                    });
                }
                break;
            }
        }
    }

    decls
}

/// Find a keyword in a line, returning the text after it.
fn find_keyword_in_line<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    // The keyword must appear as a word boundary (preceded by nothing or whitespace)
    for (i, _) in line.match_indices(keyword) {
        if i == 0 || line.as_bytes().get(i - 1).map_or(true, |b| !b.is_ascii_alphanumeric()) {
            let after = &line[i + keyword.len()..];
            if !after.is_empty() {
                return Some(after);
            }
        }
    }
    None
}

/// Extract identifier name before the first `(`, `<`, `{`, or `:`.
fn extract_name_before_paren(s: &str) -> &str {
    let s = s.trim();
    let end = s
        .find(|c: char| c == '(' || c == '<' || c == '{' || c == ':')
        .unwrap_or(s.len());
    s[..end].trim()
}

/// Extract identifier name before the first `(`, `<`, `{`, `:`, or whitespace.
fn extract_name_before_delimiter(s: &str) -> &str {
    let s = s.trim();
    let end = s
        .find(|c: char| c == '(' || c == '<' || c == '{' || c == ':' || c == ' ' || c == '\t')
        .unwrap_or(s.len());
    s[..end].trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_camel_case() {
        assert_eq!(split_identifier_words("userId"), vec!["user", "id"]);
    }

    #[test]
    fn split_snake_case() {
        assert_eq!(split_identifier_words("user_id"), vec!["user", "id"]);
    }

    #[test]
    fn split_pascal_case() {
        assert_eq!(split_identifier_words("TrustGate"), vec!["trust", "gate"]);
    }

    #[test]
    fn split_acronym() {
        assert_eq!(split_identifier_words("HTTPClient"), vec!["http", "client"]);
    }

    #[test]
    fn extract_name_before_paren_works() {
        assert_eq!(extract_name_before_paren("handle_event(self)"), "handle_event");
        assert_eq!(extract_name_before_paren("MyType<T>"), "MyType");
        assert_eq!(extract_name_before_paren("foo"), "foo");
    }
}
