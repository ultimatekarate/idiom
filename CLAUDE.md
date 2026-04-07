# Idiom — Project Conventions

## What Is Idiom

Local convention governance. Idiom infers naming patterns from sibling files in a directory and enforces them against new code. It is the fine-grid complement to Basis (global architectural governance).

## Repository Layout

```
src/
  spec.rs, pattern.rs         Dictionary — inert types (pure, no IO)
  analyze.rs, report.rs       Laboratory — pure logic (no IO)
  loader.rs                   Rules-loader — .idiom.yaml discovery
  scan/                       Scan-engine — file walking, orchestration
  language/                   Language parsers — name extraction for 9 languages
  main.rs                     CLI runner — infer, check, context
tests/
  integration_tests.rs        End-to-end tests with temp directories
basis.yaml                    Basis governs Idiom
```

## Idiom Is Governed by Basis

This codebase is governed by `basis.yaml`. All code must pass `basis check` before it is considered complete. The six-layer architecture (dictionary, laboratory, rules-loader, scan-engine, cli-runner, language-server) is enforced at the import level.

**Dictionary and laboratory are strictly pure.** No filesystem, no IO, no async. Pattern extraction and analysis are deterministic functions.

## Coding Standards

- Rust: stable toolchain, `cargo fmt`, `cargo clippy`
- All tests must pass: `cargo test`
- Basis governance must pass: `basis check --spec basis.yaml .`
- When adding a new language, follow the existing pattern in `src/language/` — implement `extract_names` and register in `LangRegistry::new()`

## CLI Commands

```
idiom infer <directory>           # Extract and display local conventions
idiom check <file>                # Validate file against sibling conventions
idiom context <directory>         # Emit convention summary for agent injection
```

## Error Codes

| Code | Role | Meaning |
|------|------|---------|
| `I001` | Function | Function name deviates from local convention |
| `I002` | Type | Type name deviates from local convention |
| `I003` | Module | Module name deviates from local convention |
| `I004` | Constructor | Constructor name deviates from local convention |
