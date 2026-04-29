use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static GO: LangDef = LangDef {
    extensions: &["go"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // func declarations: func Foo(), func (r *Receiver) Foo()
        if let Some(after_func) = trimmed.strip_prefix("func ") {
            // Method with receiver: func (r *Type) Name(...)
            let name_part = if after_func.starts_with('(') {
                // Skip past receiver
                after_func
                    .find(')')
                    .and_then(|i| after_func.get(i + 1..))
                    .map(|s| s.trim())
                    .unwrap_or("")
            } else {
                after_func
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

        // type declarations: type Foo struct, type Foo interface
        if let Some(after_type) = trimmed.strip_prefix("type ") {
            let name = after_type
                .split_whitespace()
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
    fn extract_go_functions() {
        let code = "func HandleEvent() error {\n}\nfunc HandleMessage() {\n}\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "HandleEvent");
    }

    #[test]
    fn extract_go_methods() {
        let code = "func (g *Gate) HandleEvent() error {\n}\n";
        let decls = extract_names(code);
        assert_eq!(decls[0].name, "HandleEvent");
    }

    #[test]
    fn extract_go_types() {
        let code = "type TrustGate struct {\n}\ntype Handler interface {\n}\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
    }
}
