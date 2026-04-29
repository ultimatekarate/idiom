use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static PYTHON: LangDef = LangDef {
    extensions: &["py"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Function declarations
        if let Some(after_keyword) = trimmed
            .strip_prefix("async def ")
            .or_else(|| trimmed.strip_prefix("def "))
        {
            let name = after_keyword
                .split('(')
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

        // Class declarations
        if let Some(after_class) = trimmed.strip_prefix("class ") {
            let name = after_class
                .split(&['(', ':'][..])
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
    fn extract_python_functions() {
        let code = "def handle_event(self):\n    pass\ndef handle_message():\n    pass\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handle_event");
    }

    #[test]
    fn extract_python_classes() {
        let code = "class TrustGate:\n    pass\nclass IntegrityGate(Base):\n    pass\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "TrustGate");
        assert_eq!(types[1].name, "IntegrityGate");
    }

    #[test]
    fn skip_private_functions() {
        let code = "def _private():\n    pass\ndef public():\n    pass\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "public");
    }
}
