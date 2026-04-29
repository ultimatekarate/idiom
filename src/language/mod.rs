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
use std::collections::HashMap;

/// A language-specific name extractor.
pub struct LangDef {
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
}
