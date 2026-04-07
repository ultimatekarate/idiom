use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static JAVA: LangDef = LangDef {
    name: "java",
    extensions: &["java"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // Class/interface/enum declarations
        for kw in &["class ", "interface ", "enum "] {
            if let Some(pos) = trimmed.find(kw) {
                let prefix = &trimmed[..pos];
                // Must be preceded by access modifier, abstract, etc. or start of line
                if prefix.is_empty()
                    || prefix.ends_with(' ')
                    || prefix.ends_with('\t')
                {
                    let after_kw = &trimmed[pos + kw.len()..];
                    let name = after_kw
                        .split(&['<', '{', ' ', '('][..])
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

        // Method declarations: access type name(
        // Pattern: line contains '(' and looks like a method declaration
        if trimmed.contains('(')
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("switch ")
            && !trimmed.starts_with("return ")
            && !trimmed.starts_with("new ")
            && !trimmed.contains(" class ")
            && !trimmed.contains(" interface ")
            && !trimmed.starts_with("@")
            && !trimmed.starts_with("import ")
            && !trimmed.starts_with("package ")
        {
            // Try to extract method name: the word immediately before '('
            if let Some(paren_pos) = trimmed.find('(') {
                let before_paren = trimmed[..paren_pos].trim();
                let name = before_paren
                    .rsplit_once(' ')
                    .map(|(_, n)| n)
                    .unwrap_or(before_paren);
                // Filter out keywords and constructors (PascalCase check)
                if !name.is_empty()
                    && !name.starts_with('_')
                    && name != "class"
                    && name != "interface"
                    && name != "enum"
                    && name != "if"
                    && name != "for"
                    && name != "while"
                    && name.chars().next().map_or(false, |c| c.is_lowercase())
                {
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
    fn extract_java_types() {
        let code = "public class TrustGate {\n}\npublic interface Handler {\n}\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "TrustGate");
    }

    #[test]
    fn extract_java_methods() {
        let code = "    public void handleEvent(String data) {\n    }\n    private int handleMessage() {\n    }\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handleEvent");
    }
}
