use std::process::Command;

fn run(args: &[&str], code: i32, message: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_idealis"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/specs"))
        .args(args)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(code), "{text}");
    assert!(text.contains(message), "{text}");
    text
}
#[test]
fn finite_tlc_checks_the_selected_configuration() {
    run(
        &["tlc", "-workers", "1", "Counter"],
        0,
        "No error has been found",
    );
    run(&["tlc", "Counter.tla", "-config", "Bad"], 12, "Broken");
    run(&["tlc", "-simulate", "Counter"], 1, "-simulate");
}
#[test]
fn sany_distinguishes_parsing_and_semantics() {
    run(
        &["tla", "Counter"],
        0,
        "Semantic processing of module Counter",
    );
    run(&["tla", "Unknown.tla"], 0, "Unknown operator: `Missing'.");
    run(&["tla", "-s", "Unknown.tla"], 0, "Parsing file");
    run(&["tla", "-d", "Counter"], 255, "-d");
}

#[test]
fn checks_local_imports_and_deadlock_switch() {
    run(&["tlc", "Stop"], 11, "Deadlock reached.");
    run(&["tlc", "-deadlock", "Stop"], 0, "No error has been found");
    run(&["tla", "Stop"], 0, "Semantic processing of module Stop");
}

#[test]
fn incomplete_search_never_reports_success() {
    run(
        &["tlc", "-config", "Bound", "Counter"],
        255,
        "bounded (depth)",
    );
    run(&["tlc", "-config", "Empty", "Counter"], 255, "empty");
    run(&["tlc", "MissingModel"], 255, "MissingModel.cfg");
}

#[test]
fn lowering_cannot_discard_an_unchecked_theorem() {
    run(&["tlc", "-config", "Stop", "Proof"], 150, "proofs");
    run(&["tla", "Proof"], 255, "proofs");
    run(&["tla", "-s", "Proof"], 0, "Parsing file");
}

#[test]
fn tlc_prints_difftrace_with_full_error_state() {
    let report = run(
        &[
            "tlc",
            "-modelcheck",
            "-workers",
            "auto",
            "-difftrace",
            "Diff",
        ],
        12,
        "State 3:",
    );
    assert_eq!(report.matches("stable = \"constant\"").count(), 2);
    let middle = report
        .split_once("State 2:")
        .unwrap()
        .1
        .split_once("State 3:")
        .unwrap()
        .0;
    assert!(middle.contains("x = 1"));
    assert!(!middle.contains("stable"));
    run(&["tlc", "-h"], 1, "-difftrace");
}

#[test]
fn sany_reports_level_diagnostics_and_honors_the_level_switch() {
    run(&["tla", "-stat", "Level"], 0, "Semantic errors:");
    let output = run(&["tla", "-l", "Level"], 0, "Semantic processing");
    assert!(!output.contains("Semantic errors:"));
}

#[test]
fn false_constant_checks_are_configuration_errors() {
    for cfg in ["Rejected", "Property"] {
        run(&["tlc", "-config", cfg, "Values"], 151, "is equal to FALSE");
    }
}
