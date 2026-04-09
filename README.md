# Idiom

Local naming-convention governance for your codebase. Idiom infers naming patterns from sibling files in a directory and enforces them against new code.

## Why Idiom?

Linters enforce language-wide style rules. Idiom enforces **your** style — the patterns that already exist in each directory. If every handler in `src/handlers/` starts with `handle_`, Idiom notices and flags `process_data` as a deviation. No manual configuration required.

## Quick Start

```bash
cargo install --path .

# See what conventions exist in a directory
idiom infer src/handlers/

# Check a file against its sibling conventions
idiom check src/handlers/new_handler.rs

# Generate a convention summary for AI agent context
idiom context src/handlers/
```

## How It Works

1. **Scan** — Idiom walks the target directory and extracts named declarations (functions, types, modules, constructors) using language-aware parsers.
2. **Infer** — It detects patterns across those declarations: common prefixes (`handle_`), suffixes (`Gate`), and casing (`snake_case`, `PascalCase`, etc.).
3. **Enforce** — High-confidence patterns (100% sibling agreement) are enforced by default. Medium/low-confidence patterns can be enforced via overrides.

## Supported Languages

| Language | Extensions |
| ---------- | ----------- |
| Rust | `.rs` |
| Python | `.py` |
| JavaScript | `.js`, `.jsx` |
| TypeScript | `.ts`, `.tsx` |
| Go | `.go` |
| Java | `.java` |
| Kotlin | `.kt`, `.kts` |
| Ruby | `.rb` |
| Swift | `.swift` |
| C# | `.cs` |

## CLI Commands

### `idiom infer <directory>`

Analyzes sibling files and reports detected naming patterns.

```bash
$ idiom infer ./src/handlers/

Conventions inferred for: ./src/handlers/

  function prefix: prefix 'handle_' (High confidence, 3/3 match)
  type suffix: suffix 'Gate' (High confidence, 3/3 match)
  function casing: snake_case (High confidence, 3/3 match)
```

### `idiom check <file>`

Validates a file against its directory's conventions. Exits non-zero on deviations.

```bash
$ idiom check src/handlers/data.rs

warning[I001]: function name 'process_data' does not match local convention: prefix 'handle_'
  --> src/handlers/data.rs:42
  = note: local convention is prefix 'handle_'
  = help: rename to 'handle_data'

error: aborting due to 1 idiom deviation(s)
```

### `idiom context <directory>`

Emits a convention summary suitable for injecting into AI agent prompts.

```bash
$ idiom context ./src/handlers/

# Local conventions for ./src/handlers/

When writing code in this directory, follow these naming conventions:

- function names should start with 'handle_' (e.g., handle_event)
- type names should end with 'Gate' (e.g., TrustGate)
```

All commands accept `--format json` for machine-readable output.

## Error Codes

| Code | Role | Meaning |
| ------ | ------ | --------- |
| `I001` | Function | Function name deviates from local convention |
| `I002` | Type | Type name deviates from local convention |
| `I003` | Module | Module name deviates from local convention |
| `I004` | Constructor | Constructor name deviates from local convention |

## Overrides with `.idiom.yaml`

Place an `.idiom.yaml` file in any directory to pin or suppress pattern enforcement.

```yaml
naming:
  functions:
    pin:
      prefix: "handle_"      # Enforce this prefix regardless of confidence
      casing: "snake_case"
    suppress:
      suffix: true            # Don't enforce suffix patterns
  types:
    pin:
      suffix: "Gate"
  constructors:
    suppress:
      prefix: true
```

- **`pin`** — Force enforcement of a pattern value, even at low confidence.
- **`suppress`** — Disable enforcement of a pattern kind entirely.

## Building

```bash
cargo build --release
cargo test
cargo clippy
```

## License

See [LICENSE](LICENSE) for details.
