use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static JAVASCRIPT: LangDef = LangDef {
    extensions: &["js", "ts", "jsx", "tsx"],
    extract_names,
};

fn extract_names(content: &str) -> Vec<NamedDecl> {
    let mut decls = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // function declarations: function foo(), export function foo(), async function foo()
        if let Some(pos) = trimmed.find("function ") {
            let prefix = &trimmed[..pos];
            if prefix.is_empty()
                || prefix.trim_end().ends_with("export")
                || prefix.trim_end().ends_with("async")
                || prefix.trim_end().ends_with("default")
            {
                let after_fn = &trimmed[pos + 9..];
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
                }
            }
        }

        // Arrow function assigned to const: const foo = (...) =>
        // Also catches: export const foo = (...) =>
        if let Some(after_const) = trimmed
            .strip_prefix("export const ")
            .or_else(|| trimmed.strip_prefix("const "))
        {
            if trimmed.contains("=>") || trimmed.contains("= (") || trimmed.contains("= async") {
                let name = after_const
                    .split(&['=', ':', ' '][..])
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

        // Class/interface/type declarations
        if let Some(after_kw) = trimmed
            .strip_prefix("export class ")
            .or_else(|| trimmed.strip_prefix("export interface "))
            .or_else(|| trimmed.strip_prefix("export type "))
            .or_else(|| trimmed.strip_prefix("class "))
            .or_else(|| trimmed.strip_prefix("interface "))
            .or_else(|| trimmed.strip_prefix("type "))
        {
            let name = after_kw
                .split(&['<', '{', ' ', '=', '('][..])
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
    fn extract_js_functions() {
        let code = "function handleEvent() {}\nexport function handleMessage() {}\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "handleEvent");
    }

    #[test]
    fn extract_ts_types() {
        let code = "interface TrustGate {}\nexport type SignalKind = string;\n";
        let decls = extract_names(code);
        let types: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Type)
            .collect();
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn extract_arrow_functions() {
        let code = "const handleEvent = () => {};\nexport const handleMessage = async () => {};\n";
        let decls = extract_names(code);
        let fns: Vec<_> = decls
            .iter()
            .filter(|d| d.role == SyntacticRole::Function)
            .collect();
        assert_eq!(fns.len(), 2);
    }
}
