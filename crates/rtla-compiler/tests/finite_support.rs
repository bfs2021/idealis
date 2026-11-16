use rtlc::SafetyBounds;
use serde_json::Value;

fn verify(family: &str) {
    let cases: Vec<Value> = serde_json::from_str(include_str!("finite-support.json")).unwrap();
    for case in cases.iter().filter(|case| case["family"] == family) {
        let name = case["name"].as_str().unwrap();
        let module = rtla::parse_module(case["source"].as_str().unwrap()).unwrap();
        let config = rtla::parse_config(case["config"].as_str().unwrap()).unwrap();
        let program = rtla_compiler::compile(&module, &config)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        for workers in [1, 4] {
            let result = program.check_parallel(
                SafetyBounds {
                    max_depth: 100,
                    max_states: 1000,
                    max_work: 1_000_000,
                    max_temporal_nodes: Some(10000),
                },
                workers,
            );
            if case["expected"] == "model-error" {
                let error = result.expect_err(name);
                assert!(
                    error
                        .message
                        .to_lowercase()
                        .contains(case["error"].as_str().unwrap()),
                    "{name}: {error}"
                );
            } else {
                let result = result.unwrap_or_else(|error| panic!("{name}: {error}"));
                assert_eq!(result.status, case["expected"], "{name}");
                if name == "state-constraint" || name == "action-constraint" {
                    assert_eq!(result.states, 3, "{name}");
                    assert_eq!(result.generated, 4, "{name}");
                }
            }
        }
    }
}

#[test]
fn integer_operations_match_tlc() {
    verify("arithmetic");
}
#[test]
fn bounded_choose_matches_tlc() {
    verify("choose");
}
#[test]
fn constraints_preserve_boundary_checks() {
    verify("constraints");
}
#[test]
fn constant_temporal_quantifiers_match_tlc() {
    verify("temporal");
}
#[test]
fn model_values_preserve_identity_and_types() {
    verify("models");
}

#[test]
fn choice_and_temporal_expansion_consume_the_work_budget() {
    for body in ["CHOOSE n \\in 1..1000000: FALSE", "0"] {
        let source = format!(
            "---- MODULE Budget ----\nEXTENDS Integers\nVARIABLE x\nInit == x = {body}\nNext == UNCHANGED x\nLive == \\A n \\in 1..1000000: <> (x = n)\n===="
        );
        let config = if body == "0" {
            "INIT Init\nNEXT Next\nPROPERTY Live"
        } else {
            "INIT Init\nNEXT Next"
        };
        let program = rtla_compiler::compile(
            &rtla::parse_module(&source).unwrap(),
            &rtla::parse_config(config).unwrap(),
        )
        .unwrap();
        let result = program
            .check(SafetyBounds {
                max_depth: 10,
                max_states: 100,
                max_work: 100,
                max_temporal_nodes: Some(100),
            })
            .unwrap();
        assert_eq!(result.status, "bounded");
        assert_eq!(result.reason.as_deref(), Some("work"));
        assert_eq!(result.work, 100);
    }
}

#[test]
fn excluded_constraint_candidates_stay_within_the_depth_budget() {
    let source = "---- MODULE Depth ----\nEXTENDS Integers\nVARIABLE x\nInit == x = 0\nNext == x' = x + 1\nLimit == x < 1\nSafe == x < 1\n====";
    let program = rtla_compiler::compile(
        &rtla::parse_module(source).unwrap(),
        &rtla::parse_config("INIT Init\nNEXT Next\nCONSTRAINT Limit\nINVARIANT Safe").unwrap(),
    )
    .unwrap();
    let result = program
        .check(SafetyBounds {
            max_depth: 0,
            max_states: 10,
            max_work: 1000,
            max_temporal_nodes: None,
        })
        .unwrap();
    assert_eq!(result.status, "bounded");
    assert_eq!(result.reason.as_deref(), Some("depth"));
    assert!(result.violation.is_none());
}

#[test]
fn temporal_domains_must_be_constant_and_ir_slots_remain_scoped() {
    let source = "---- MODULE Scope ----\nEXTENDS Integers\nVARIABLE x\nInit == x = 0\nNext == UNCHANGED x\nLive == \\A n \\in {x}: <> (x = n)\n====";
    assert!(
        rtla_compiler::compile(
            &rtla::parse_module(source).unwrap(),
            &rtla::parse_config("INIT Init\nNEXT Next\nPROPERTY Live").unwrap()
        )
        .unwrap_err()
        .message
        .contains("constant finite")
    );
    let module = rtla::parse_module(&source.replace("{x}", "{0}")).unwrap();
    let program = rtla_compiler::compile(
        &module,
        &rtla::parse_config("INIT Init\nNEXT Next\nPROPERTY Live").unwrap(),
    )
    .unwrap();
    let mut input = serde_json::to_value(program).unwrap();
    input["properties"][0]["formula"]["bounds"][0]["slot"] = serde_json::json!(1);
    let invalid: rtlc::SafetyProgram = serde_json::from_value(input).unwrap();
    assert!(
        invalid
            .check(SafetyBounds {
                max_depth: 10,
                max_states: 100,
                max_work: 1000,
                max_temporal_nodes: None
            })
            .unwrap_err()
            .message
            .contains("binding slot")
    );
}
