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
            "INIT Init\nNEXT Next\nCONSTRAINT Missing",
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
#[test]
fn fold_rejects_a_recursive_operator_in_the_ast() {
    let mut module = rtla::parse_module("---- MODULE Fold ----\nEXTENDS Apalache\nVARIABLE x\nOp(a, b) == a\nInit == x = ApaFoldSet(Op, 0, {})\nNext == UNCHANGED x\n====").unwrap();
    module.definitions[0].is_recursive = true;
    assert!(
        compile(
            &module,
            &rtla::parse_config("INIT Init\nNEXT Next").unwrap()
        )
        .is_err()
    );
}

#[test]
fn apalache_assignment_requires_the_module_and_preserves_guards() {
    let source = "---- MODULE Assignment ----\nEXTENDS Integers, Apalache\nVARIABLE x\nInit == x := 0\nNext == x' := x + 1\nSafe == (x := 0) \\/ (x := 1)\n====";
    let config = "INIT Init\nNEXT Next\nINVARIANT Safe";
    let result = program(source, config)
        .check(SafetyBounds {
            max_depth: 10,
            max_states: 100,
            max_work: 100000,
            max_temporal_nodes: None,
        })
        .unwrap();
    let result = serde_json::to_value(result).unwrap();
    assert_eq!(result["status"], "invariant-violation");
    assert_eq!(result["violation"]["trace"].as_array().unwrap().len(), 3);
    assert!(
        compile(
            &rtla::parse_module(&source.replace("Integers, Apalache", "Integers")).unwrap(),
            &rtla::parse_config(config).unwrap(),
        )
        .is_err()
    );
}

#[test]
fn variants_preserve_payload_scope_and_reject_ambiguous_tags() {
    let source = "---- MODULE VariantPayload ----\nEXTENDS Apalache, Integers, Variants\nVARIABLE x\nAdd(acc, element) == acc + element\nInit == x = Variant(\"Outer\", ApaFoldSeqLeft(Add, 0, <<1, 2>>))\nNext == UNCHANGED x\nSafe == VariantTag(x) = \"Outer\" /\\ VariantGetUnsafe(\"Outer\", x) = 3\n====";
    let config = "INIT Init\nNEXT Next\nINVARIANT Safe";
    let bounds = SafetyBounds {
        max_depth: 10,
        max_states: 100,
        max_work: 100000,
        max_temporal_nodes: None,
    };
    assert_eq!(
        serde_json::to_value(program(source, config).check(bounds).unwrap()).unwrap()["status"],
        "complete"
    );
    assert!(
        compile(
            &rtla::parse_module(&source.replace(", Variants", "")).unwrap(),
            &rtla::parse_config(config).unwrap(),
        )
        .is_err()
    );
    for value in [
        "[tag \\in {} |-> 0]",
        "[first |-> 1, second |-> 2]",
        "<<1>>",
    ] {
        let invalid = format!(
            "---- MODULE InvalidTag ----\nEXTENDS Variants\nVARIABLE x\nInit == x = VariantTag({value})\nNext == UNCHANGED x\n===="
        );
        assert!(
            program(&invalid, "INIT Init\nNEXT Next")
                .check(bounds)
                .unwrap_err()
                .message
                .contains("singleton string domain")
        );
    }
}
