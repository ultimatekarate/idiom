use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static KOTLIN: LangDef = LangDef {
    name: "kotlin",
    extensions: &["kt", "kts"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // fun declarations
        if let Some(pos) = trimmed.find("fun ") {
            let prefix = &trimmed[..pos];
            if prefix.is_empty()
                || prefix.ends_with(' ')
                || prefix.ends_with('\t')
            {
                let after_fun = &trimmed[pos + 4..];
                // Skip extension functions receiver: Type.name(
                let name_part = if let Some(dot_pos) = after_fun.find('.') {
                    let before_dot = &after_fun[..dot_pos];
                    if !before_dot.contains('(') && !before_dot.contains(' ') {
                        &after_fun[dot_pos + 1..]
                    } else {
                        after_fun
                    }
                } else {
                    after_fun
                };
                let name = name_part
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

        // class/interface/object/data class/sealed class
        for kw in &["class ", "interface ", "object "] {
            if let Some(pos) = trimmed.find(kw) {
                let prefix = &trimmed[..pos];
                if prefix.is_empty()
                    || prefix.ends_with(' ')
                    || prefix.ends_with('\t')
                {
                    let after_kw = &trimmed[pos + kw.len()..];
                    let name = after_kw
                        .split(&['<', '{', '(', ' ', ':'][..])
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
    fn extract_kotlin_functions() {
        let code = "fun handleEvent() {\n}\nsuspend fun handleMessage(): Unit {\n}\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handleEvent");
    }

    #[test]
    fn extract_kotlin_types() {
        let code = "data class TrustGate(\n)\nsealed interface SignalKind {\n}\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
    }
}
