use rtla_compiler::compile_modules;
use rtlc::SafetyBounds;

#[test]
fn fixed_module_corpus_preserves_results_and_rejections() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("modules-support.json")).unwrap();
    for case in cases {
        let modules = case["modules"]
            .as_object()
            .unwrap()
            .values()
            .map(|source| rtla::parse_module(source.as_str().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let compiled = compile_modules(
            &rtla::parse_module(case["source"].as_str().unwrap()).unwrap(),
            &rtla::parse_config(case["config"].as_str().unwrap()).unwrap(),
            &modules,
        );
        if case["expected"] == "unsupported" {
            assert!(compiled.is_err(), "{}", case["name"]);
            continue;
        }
        let result = compiled
            .unwrap_or_else(|error| panic!("{}: {error}", case["name"]))
            .check(SafetyBounds {
                max_depth: 20,
                max_states: 1000,
                max_work: 1_000_000,
                max_temporal_nodes: Some(10000),
            });
        if case["expected"] == "model-error" {
            assert!(result.is_err(), "{}", case["name"]);
            continue;
        }
        let result = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(result["status"], case["expected"], "{}", case["name"]);
        if let Some(action) = case.get("action") {
            assert_eq!(
                &result["violation"]["trace"][1]["action"], action,
                "{}",
                case["name"]
            );
        }
        if let Some(states) = case.get("states") {
            assert_eq!(&result["states"], states, "{}", case["name"]);
        }
    }
}

fn check(source: &str, config: &str, modules: &[&str]) -> serde_json::Value {
    let modules = modules
        .iter()
        .map(|source| rtla::parse_module(source).unwrap())
        .collect::<Vec<_>>();
    let program = compile_modules(
        &rtla::parse_module(source).unwrap(),
        &rtla::parse_config(config).unwrap(),
        &modules,
    )
    .unwrap();
    serde_json::to_value(
        program
            .check(SafetyBounds {
                max_depth: 20,
                max_states: 1000,
                max_work: 1_000_000,
                max_temporal_nodes: Some(10000),
            })
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn external_definitions_keep_their_scope_and_local_helpers() {
    let result = check(
        "---- MODULE Root ----\nEXTENDS Counter\nSafe == x < 3\n====",
        "INIT Init\nNEXT Next\nINVARIANT Safe",
        &[
            "---- MODULE Counter ----\nEXTENDS Naturals\nVARIABLE x\nLOCAL Add(v) == v + 1\nInit == x = 0\nNext == x' = Add(x)\n====",
        ],
    );
    assert_eq!(result["status"], "invariant-violation");
    assert_eq!(result["violation"]["trace"].as_array().unwrap().len(), 4);
    assert_eq!(result["violation"]["trace"][1]["action"], "Next");
}

#[test]
fn separate_instances_substitute_variables_and_constants() {
    let result = check(
        "---- MODULE Root ----\nEXTENDS Naturals\nVARIABLES x, y\nA == INSTANCE Counter WITH value <- x, Limit <- 1\nB == INSTANCE Counter WITH value <- y, Limit <- 2\nInit == A!Init /\\ B!Init\nNext == (A!Next /\\ UNCHANGED y) \\/ (B!Next /\\ UNCHANGED x)\nSafe == x <= 1 /\\ y <= 2\n====",
        "INIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE",
        &[
            "---- MODULE Counter ----\nEXTENDS Naturals\nCONSTANT Limit\nVARIABLE value\nInit == value = 0\nNext == value < Limit /\\ value' = value + 1\n====",
        ],
    );
    assert_eq!(result["status"], "complete");
    assert_eq!(result["states"], 6);
}

#[test]
fn config_replacements_preserve_arity_and_definition_scope() {
    let source = "---- MODULE Root ----\nEXTENDS Naturals\nCONSTANT Limit, Add(_)\nVARIABLE x\nBound == 2\nInc(v) == v + 1\nInit == x = 0\nNext == x < Limit /\\ x' = Add(x)\nSafe == x <= 2\n====";
    let result = check(
        source,
        "CONSTANTS Limit <- Bound Add <- Inc\nINIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE",
        &[],
    );
    assert_eq!(result["status"], "complete");
    assert_eq!(result["states"], 3);
    let result = check(
        &source.replace("CONSTANT Limit, Add(_)", "CONSTANT Limit\nAdd(v) == 0"),
        "CONSTANTS Limit = 2 Add(0) = 1 Add(1) = 2\nINIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE",
        &[],
    );
    assert_eq!(result["status"], "complete");
    assert_eq!(result["states"], 3);
}

#[test]
fn module_bundles_reject_duplicate_names_and_bound_expansion() {
    let root = rtla::parse_module("---- MODULE Root ----\nEXTENDS M0\n====").unwrap();
    let config = rtla::parse_config("INIT Init\nNEXT Next").unwrap();
    let duplicate = rtla::parse_module("---- MODULE Root ----\n====").unwrap();
    assert!(
        compile_modules(&root, &config, &[duplicate])
            .unwrap_err()
            .message
            .contains("duplicate")
    );
    let modules = (0..41)
        .map(|index| {
            rtla::parse_module(&format!(
                "---- MODULE M{index} ----\nEXTENDS M{}\n====",
                index + 1
            ))
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert!(
        compile_modules(&root, &config, &modules)
            .unwrap_err()
            .message
            .contains("depth")
    );
    let modules = (0..257)
        .map(|index| rtla::parse_module(&format!("---- MODULE M{index} ----\n====")).unwrap())
        .collect::<Vec<_>>();
    assert!(
        compile_modules(&root, &config, &modules)
            .unwrap_err()
            .message
            .contains("256 modules")
    );
}

#[test]
fn manually_constructed_ast_depth_is_bounded_before_linking() {
    let mut module = rtla::parse_module(
        "---- MODULE Root ----\nVARIABLE x\nUnused == 0\nInit == x = 0\nNext == UNCHANGED x\n====",
    )
    .unwrap();
    for _ in 0..80 {
        module.definitions[0].body = rtla::ast::Expression {
            location: module.location,
            node: rtla::ast::ExpressionKind::Tuple {
                elements: vec![module.definitions[0].body.clone()],
            },
        };
    }
    let error = compile_modules(
        &module,
        &rtla::parse_config("INIT Init\nNEXT Next").unwrap(),
        &[],
    )
    .unwrap_err();
    assert!(error.message.contains("depth"));
}
