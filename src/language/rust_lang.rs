use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static RUST: LangDef = LangDef {
    name: "rust",
    extensions: &["rs"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    // Strip test sections
    let content = if let Some(pos) = content.find("#[cfg(test)]") {
        &content[..pos]
    } else {
        content
    };

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Function declarations
        if trimmed.starts_with("pub fn ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("pub async fn ")
            || trimmed.starts_with("async fn ")
        {
            let fn_pos = trimmed.find("fn ").unwrap_or(0) + 3;
            let after_fn = &trimmed[fn_pos..];
            let name = after_fn
                .split(&['(', '<'][..])
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() && !name.starts_with('_') {
                decls.push(NamedDecl {
                    name: name.to_string(),
                    role: SyntacticRole::Function,
                    line: line_num + 1,
                });

                // Detect constructor patterns: new(), new_*(), from_*()
                if name == "new" || name.starts_with("new_") || name.starts_with("from_") {
                    decls.push(NamedDecl {
                        name: name.to_string(),
                        role: SyntacticRole::Constructor,
                        line: line_num + 1,
                    });
                }
            }
        }

        // Type declarations
        if trimmed.starts_with("pub struct ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("trait ")
        {
            let kw_end = if trimmed.contains("struct ") {
                trimmed.find("struct ").unwrap_or(0) + 7
            } else if trimmed.contains("enum ") {
                trimmed.find("enum ").unwrap_or(0) + 5
            } else {
                trimmed.find("trait ").unwrap_or(0) + 6
            };
            let after_kw = &trimmed[kw_end..];
            let name = after_kw
                .split(&['(', '<', '{', ' ', ';'][..])
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() && !name.starts_with('_') {
                decls.push(NamedDecl {
                    name: name.to_string(),
                    role: SyntacticRole::Type,
                    line: line_num + 1,
                });
            }
        }
    }

    decls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_rust_functions() {
        let code = "pub fn handle_event() {}\nfn handle_message() {}\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handle_event");
        assert_eq!(fns[1].name, "handle_message");
    }

    #[test]
    fn extract_rust_types() {
        let code = "pub struct TrustGate {}\npub enum SignalKind {}\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "TrustGate");
        assert_eq!(types[1].name, "SignalKind");
    }

    #[test]
    fn extract_constructors() {
        let code = "pub fn new_verified() {}\npub fn from_bytes() {}\npub fn new() {}\n";
        let decls = extract_names(code);
        let ctors: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Constructor)
            .collect();
        assert_eq!(ctors.len(), 3);
    }

    #[test]
    fn skips_test_section() {
        let code = "pub fn handle_event() {}\n#[cfg(test)]\nmod tests {\n    fn test_foo() {}\n}\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "handle_event");
    }
}
