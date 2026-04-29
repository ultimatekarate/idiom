use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static SWIFT: LangDef = LangDef {
    extensions: &["swift"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // func declarations
        if let Some(pos) = trimmed.find("func ") {
            let prefix = &trimmed[..pos];
            if prefix.is_empty()
                || prefix.ends_with(' ')
                || prefix.ends_with('\t')
            {
                let after_func = &trimmed[pos + 5..];
                let name = after_func
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
                }
            }
        }

        // class/struct/enum/protocol declarations
        for kw in &["class ", "struct ", "enum ", "protocol "] {
            if let Some(pos) = trimmed.find(kw) {
                let prefix = &trimmed[..pos];
                if prefix.is_empty()
                    || prefix.ends_with(' ')
                    || prefix.ends_with('\t')
                {
                    let after_kw = &trimmed[pos + kw.len()..];
                    let name = after_kw
                        .split(&['<', '{', ':', ' '][..])
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
                    break;
                }
            }
        }
    }

    decls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_swift_functions() {
        let code = "func handleEvent() {\n}\npublic func handleMessage() -> Void {\n}\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handleEvent");
    }

    #[test]
    fn extract_swift_types() {
        let code = "class TrustGate {\n}\nstruct SignalKind {\n}\nprotocol Handler {\n}\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 3);
    }
}
