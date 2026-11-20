use std::process::Command;

fn run(args: &[&str], code: i32, message: &str) {
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
