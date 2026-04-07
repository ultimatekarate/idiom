use super::LangDef;
use crate::analyze::NamedDecl;
use crate::pattern::SyntacticRole;

pub static JAVASCRIPT: LangDef = LangDef {
    name: "js",
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
        if (trimmed.starts_with("const ") || trimmed.starts_with("export const "))
            && (trimmed.contains("=>") || trimmed.contains("= (") || trimmed.contains("= async"))
        {
            let after_const = if trimmed.starts_with("export const ") {
                &trimmed[13..]
            } else {
                &trimmed[6..]
            };
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

        // Class/interface/type declarations
        if trimmed.starts_with("class ")
            || trimmed.starts_with("export class ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("export interface ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("export type ")
        {
            let after_kw = if trimmed.starts_with("export class ") {
                &trimmed[13..]
            } else if trimmed.starts_with("export interface ") {
                &trimmed[17..]
            } else if trimmed.starts_with("export type ") {
                &trimmed[12..]
            } else if trimmed.starts_with("class ") {
                &trimmed[6..]
            } else if trimmed.starts_with("interface ") {
                &trimmed[10..]
            } else if trimmed.starts_with("type ") {
                &trimmed[5..]
            } else {
                continue;
            };
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
