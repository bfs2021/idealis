use rtla_compiler::compile;
use rtlc::SafetyBounds;

fn program(source: &str, config: &str) -> rtlc::SafetyProgram {
    compile(
        &rtla::parse_module(source).unwrap(),
        &rtla::parse_config(config).unwrap(),
    )
    .unwrap()
}

#[test]
fn source_produces_a_reachable_counterexample() {
    let compiled = program(
        "---- MODULE Counter ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x < 3 /\\ x' = x + 1\nSafe == x < 3\n====",
        "INIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE",
    );
    let result = compiled
        .check(SafetyBounds {
            max_depth: 10,
            max_states: 100,
            max_work: 100000,
            max_temporal_nodes: None,
        })
        .unwrap();
    let value = serde_json::to_value(result).unwrap();
    assert_eq!(value["status"], "invariant-violation");
    assert_eq!(value["violation"]["trace"].as_array().unwrap().len(), 4);
}

#[test]
fn lexical_bindings_are_captured_at_definition() {
    let compiled = program(
        "---- MODULE Lexical ----\nEXTENDS Naturals\nVARIABLE x\nId(v) == v\nInit == x = 0\nNext == LET A(v) == LET B == v + 1 IN B IN x' = A(x)\nSafe == x < 2\n====",
        "INIT Init\nNEXT Next\nINVARIANT Safe",
    );
    let result = compiled
        .check(SafetyBounds {
            max_depth: 3,
            max_states: 10,
            max_work: 10000,
            max_temporal_nodes: None,
        })
        .unwrap();
    assert_eq!(
        serde_json::to_value(result).unwrap()["status"],
        "invariant-violation"
    );
}

#[test]
fn unsupported_inputs_do_not_become_a_successful_model() {
    for (source, config) in [
        (
            "---- MODULE M ----\nVARIABLE x\nInit == x = 0\nNext == TRUE\n====",
            "INIT Init\nNEXT Next",
        ),
        (
            "---- MODULE M ----\nCONSTANT N\nVARIABLE x\nInit == x = N\nNext == UNCHANGED x\n====",
            "INIT Init\nNEXT Next",
        ),
        (
            "---- MODULE M ----\nVARIABLE x\nInit == x = 0\nNext == UNCHANGED x\n====",
            "INIT Init\nNEXT Next\nCONSTRAINT Init",
        ),
    ] {
        assert!(
            compile(
                &rtla::parse_module(source).unwrap(),
                &rtla::parse_config(config).unwrap()
            )
            .is_err()
        );
    }
}

#[test]
fn fairness_and_temporal_properties_survive_lowering() {
    let compiled = program(
        "---- MODULE Ring ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x' = 1 - x\nSpec == Init /\\ [][Next]_x /\\ WF_x(Next)\nLive == []<>(x = 1)\n====",
        "SPECIFICATION Spec\nPROPERTY Live",
    );
    let result = compiled
        .check(SafetyBounds {
            max_depth: 10,
            max_states: 100,
            max_work: 100000,
            max_temporal_nodes: None,
        })
        .unwrap();
    assert_eq!(serde_json::to_value(result).unwrap()["status"], "complete");
}

#[test]
fn empty_initial_domain_stays_empty() {
    let compiled = program(
        "---- MODULE Empty ----\nVARIABLE x\nInit == FALSE\nNext == UNCHANGED x\n====",
        "INIT Init\nNEXT Next",
    );
    let result = compiled
        .check(SafetyBounds {
            max_depth: 1,
            max_states: 10,
            max_work: 1000,
            max_temporal_nodes: None,
        })
        .unwrap();
    assert_eq!(serde_json::to_value(result).unwrap()["status"], "empty");
}
