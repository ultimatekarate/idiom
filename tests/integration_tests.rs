use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn idiom_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_idiom-cli"));
    if !path.exists() {
        // Fallback for test runner
        path = PathBuf::from("target/debug/idiom-cli");
        if cfg!(windows) {
            path.set_extension("exe");
        }
    }
    path
}

fn create_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("idiom_test_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

// ── Infer Tests ─────────────────────────────────────────────────────

#[test]
fn infer_detects_snake_case_prefix() {
    let dir = create_temp_dir("snake_prefix");

    fs::write(
        dir.join("gate_trust.rs"),
        "pub fn handle_event() {}\npub fn handle_message() {}\npub fn handle_timeout() {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("handle_"),
        "Expected 'handle_' prefix in output: {stdout}"
    );

    cleanup(&dir);
}

#[test]
fn infer_detects_camel_case_prefix() {
    let dir = create_temp_dir("camel_prefix");

    fs::write(
        dir.join("handlers.js"),
        "function handleEvent() {}\nfunction handleMessage() {}\nfunction handleTimeout() {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("handle"),
        "Expected 'handle' prefix in output: {stdout}"
    );

    cleanup(&dir);
}

#[test]
fn infer_detects_pascal_suffix() {
    let dir = create_temp_dir("pascal_suffix");

    fs::write(dir.join("gates.rs"), "pub struct TrustGate {}\n").unwrap();
    fs::write(dir.join("integrity.rs"), "pub struct IntegrityGate {}\n").unwrap();
    fs::write(dir.join("topology.rs"), "pub struct TopologyGate {}\n").unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Gate"),
        "Expected 'Gate' suffix in output: {stdout}"
    );

    cleanup(&dir);
}

#[test]
fn infer_json_output() {
    let dir = create_temp_dir("json_output");

    fs::write(
        dir.join("handlers.py"),
        "def handle_event():\n    pass\ndef handle_message():\n    pass\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nOutput: {stdout}"));
    assert!(parsed.is_array());

    cleanup(&dir);
}

// ── Check Tests ─────────────────────────────────────────────────────

#[test]
fn check_passes_for_conforming_file() {
    let dir = create_temp_dir("check_pass");

    fs::write(
        dir.join("existing1.rs"),
        "pub fn handle_event() {}\npub fn handle_message() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("existing2.rs"),
        "pub fn handle_timeout() {}\npub fn handle_request() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("new_file.rs"),
        "pub fn handle_data() {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["check", dir.join("new_file.rs").to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Expected pass, got: {stdout}");
    assert!(stdout.contains("idiom check passed"));

    cleanup(&dir);
}

#[test]
fn check_detects_deviation() {
    let dir = create_temp_dir("check_deviation");

    fs::write(
        dir.join("handler1.rs"),
        "pub fn handle_event() {}\npub fn handle_message() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("handler2.rs"),
        "pub fn handle_timeout() {}\npub fn handle_request() {}\n",
    )
    .unwrap();
    // This file breaks the convention
    fs::write(
        dir.join("bad_file.rs"),
        "pub fn process_data() {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["check", dir.join("bad_file.rs").to_str().unwrap()])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for non-conforming file"
    );
    assert!(
        stderr.contains("I001"),
        "Expected I001 violation in: {stderr}"
    );
    assert!(
        stderr.contains("process_data"),
        "Expected name 'process_data' in: {stderr}"
    );

    cleanup(&dir);
}

#[test]
fn check_json_output() {
    let dir = create_temp_dir("check_json");

    fs::write(
        dir.join("handler1.rs"),
        "pub fn handle_event() {}\npub fn handle_message() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("handler2.rs"),
        "pub fn handle_timeout() {}\npub fn handle_request() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("bad_file.rs"),
        "pub fn process_data() {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args([
            "check",
            dir.join("bad_file.rs").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nOutput: {stdout}"));
    assert!(parsed["total"].as_u64().unwrap() > 0);
    assert!(parsed["deviations"].is_array());

    cleanup(&dir);
}

#[test]
fn check_excludes_target_from_inference() {
    // If target isn't excluded, its own names would bias the inference
    // and it would always pass (it matches its own pattern).
    let dir = create_temp_dir("check_exclude");

    fs::write(
        dir.join("handler1.rs"),
        "pub fn handle_event() {}\npub fn handle_message() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("handler2.rs"),
        "pub fn handle_timeout() {}\npub fn handle_request() {}\n",
    )
    .unwrap();
    // Target file has a different pattern — 2 functions with "process_" prefix
    fs::write(
        dir.join("outlier.rs"),
        "pub fn process_data() {}\npub fn process_result() {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["check", dir.join("outlier.rs").to_str().unwrap()])
        .output()
        .unwrap();

    // Should fail because siblings use handle_, not process_
    assert!(
        !output.status.success(),
        "Expected failure when target doesn't match sibling conventions"
    );

    cleanup(&dir);
}

// ── Context Tests ───────────────────────────────────────────────────

#[test]
fn context_produces_agent_instructions() {
    let dir = create_temp_dir("context");

    fs::write(
        dir.join("gates.rs"),
        "pub struct TrustGate {}\npub struct IntegrityGate {}\npub struct TopologyGate {}\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["context", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("conventions"),
        "Expected convention instructions in: {stdout}"
    );
    assert!(
        stdout.contains("Gate"),
        "Expected 'Gate' suffix mention in: {stdout}"
    );

    cleanup(&dir);
}

// ── Override Tests ───────────────────────────────────────────────────

#[test]
fn override_suppresses_pattern() {
    let dir = create_temp_dir("override_suppress");

    fs::write(
        dir.join("handler1.rs"),
        "pub fn handle_event() {}\npub fn handle_message() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("handler2.rs"),
        "pub fn handle_timeout() {}\npub fn handle_request() {}\n",
    )
    .unwrap();
    // This file breaks the convention but we suppress prefix checking
    fs::write(
        dir.join("special.rs"),
        "pub fn process_data() {}\n",
    )
    .unwrap();

    // Write override that suppresses prefix enforcement
    fs::write(
        dir.join(".idiom.yaml"),
        "naming:\n  functions:\n    suppress:\n      prefix: true\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["check", dir.join("special.rs").to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Expected pass with prefix suppressed, got stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    cleanup(&dir);
}

// ── Multi-Language Tests ────────────────────────────────────────────

#[test]
fn infer_works_for_python() {
    let dir = create_temp_dir("python");

    fs::write(
        dir.join("handlers.py"),
        "def fetch_user():\n    pass\ndef fetch_order():\n    pass\ndef fetch_product():\n    pass\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fetch_"),
        "Expected 'fetch_' prefix in output: {stdout}"
    );

    cleanup(&dir);
}

#[test]
fn infer_works_for_go() {
    let dir = create_temp_dir("golang");

    fs::write(
        dir.join("handlers.go"),
        "func HandleEvent() error { return nil }\nfunc HandleMessage() error { return nil }\nfunc HandleTimeout() error { return nil }\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Go functions are PascalCase — should detect "Handle" as camelCase prefix
    // (split_identifier_words normalizes to lowercase "handle")
    assert!(
        stdout.contains("handle") || stdout.contains("Handle"),
        "Expected Handle/handle prefix in output: {stdout}"
    );

    cleanup(&dir);
}

#[test]
fn infer_detects_snake_case_suffix() {
    let dir = create_temp_dir("snake_suffix");

    fs::write(
        dir.join("models.rs"),
        "pub fn user_id() -> u64 { 0 }\npub fn order_id() -> u64 { 0 }\npub fn product_id() -> u64 { 0 }\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["infer", dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("_id"),
        "Expected '_id' suffix in output: {stdout}"
    );

    cleanup(&dir);
}

#[test]
fn check_detects_suffix_deviation() {
    let dir = create_temp_dir("suffix_deviation");

    fs::write(
        dir.join("queries.rs"),
        "pub fn user_id() -> u64 { 0 }\npub fn order_id() -> u64 { 0 }\npub fn product_id() -> u64 { 0 }\n",
    )
    .unwrap();
    fs::write(
        dir.join("bad.rs"),
        "pub fn session_count() -> u64 { 0 }\n",
    )
    .unwrap();

    let output = Command::new(idiom_binary())
        .args(["check", dir.join("bad.rs").to_str().unwrap()])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Expected failure for suffix deviation"
    );
    assert!(
        stderr.contains("session_count"),
        "Expected 'session_count' in: {stderr}"
    );

    cleanup(&dir);
}
