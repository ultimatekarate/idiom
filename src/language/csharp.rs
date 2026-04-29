use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static CSHARP: LangDef = LangDef {
    extensions: &["cs"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // class/interface/struct/enum/record declarations
        for kw in &["class ", "interface ", "struct ", "enum ", "record "] {
            if let Some(pos) = trimmed.find(kw) {
                let prefix = &trimmed[..pos];
                if prefix.is_empty()
                    || prefix.ends_with(' ')
                    || prefix.ends_with('\t')
                {
                    let after_kw = &trimmed[pos + kw.len()..];
                    let name = after_kw
                        .split(&['<', '{', ':', ' ', '('][..])
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

        // Method declarations: look for name( pattern with return type before it
        if trimmed.contains('(')
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("switch ")
            && !trimmed.starts_with("return ")
            && !trimmed.starts_with("new ")
            && !trimmed.starts_with("using ")
            && !trimmed.starts_with("namespace ")
            && !trimmed.starts_with("[")
            && !trimmed.contains(" class ")
            && !trimmed.contains(" interface ")
            && !trimmed.contains(" struct ")
        {
            if let Some(paren_pos) = trimmed.find('(') {
                let before_paren = trimmed[..paren_pos].trim();
                let name = before_paren
                    .rsplit_once(' ')
                    .map(|(_, n)| n)
                    .unwrap_or(before_paren);
                if !name.is_empty()
                    && !name.starts_with('_')
                    && name.chars().next().is_some_and(|c| c.is_alphabetic())
                    && name != "class"
                    && name != "new"
                    && name != "if"
                    && name != "for"
                    && name != "while"
                    && name.chars().next().is_some_and(|c| c.is_uppercase())
                {
                    // C# methods are PascalCase — only capture PascalCase names
                    decls.push(NamedDecl {
                        name: name.to_string(),
                        role: SyntacticRole::Function,
                        line: line_num + 1,
                    });
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
    fn extract_csharp_types() {
        let code = "public class TrustGate {\n}\ninternal interface IHandler {\n}\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "TrustGate");
    }

    #[test]
    fn extract_csharp_methods() {
        let code = "    public void HandleEvent(string data) {\n    }\n    private int HandleMessage() {\n    }\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "HandleEvent");
    }
}
