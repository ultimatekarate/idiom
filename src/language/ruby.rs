use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static RUBY: LangDef = LangDef {
    extensions: &["rb"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // def method_name
        if let Some(after_def) = trimmed.strip_prefix("def ") {
            // Handle self.method_name for class methods
            let name_part = after_def
                .strip_prefix("self.")
                .unwrap_or(after_def);
            let name = name_part
                .split(&['(', ' ', ';'][..])
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

        // class ClassName / module ModuleName
        if trimmed.starts_with("class ") || trimmed.starts_with("module ") {
            let kw_len = if trimmed.starts_with("class ") { 6 } else { 7 };
            let after_kw = &trimmed[kw_len..];
            let name = after_kw
                .split(&['<', ' ', ';', ':'][..])
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
    fn extract_ruby_methods() {
        let code = "  def handle_event\n  end\n  def handle_message(data)\n  end\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handle_event");
    }

    #[test]
    fn extract_ruby_classes() {
        let code = "class TrustGate < Base\nend\nmodule Handlers\nend\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "TrustGate");
    }
}
