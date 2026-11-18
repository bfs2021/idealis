use rtla::ast::{Config, Module};
use rtla_compiler::{
    CompileError, CompileOptions, IrVersion, compile_modules_with_options, compile_with_options,
};
use serde_json::Value;

const V1: CompileOptions = CompileOptions {
    irVersion: IrVersion::V1,
};
const V2: CompileOptions = CompileOptions {
    irVersion: IrVersion::V2,
};
const V2_LINKING_REFUSAL: &str = "Unsupported TLA input: safety IR v2 does not support module linking or configuration operator substitution";

const COUNTER: (&str, &str, &str) = (
    "standard-extends-counter",
    "---- MODULE Counter ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x < 3 /\\ x' = x + 1\nSafe == x < 3\n====",
    "INIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE",
);
const SHARED_OPERATORS: (&str, &str, &str) = (
    "standard-extends-shared-operators",
    "---- MODULE Lexical ----\nEXTENDS Naturals\nVARIABLE x\nId(v) == v\nInit == x = 0\nNext == LET A(v) == LET B == v + 1 IN B IN x' = A(Id(x))\nSafe == x < 2\n====",
    "INIT Init\nNEXT Next\nINVARIANT Safe",
);
const UNUSED: &str = "---- MODULE Unused ----\nEXTENDS Naturals\nFoo == 1\n====";
const UNUSED_CHAIN: [&str; 2] = [
    "---- MODULE Base ----\nEXTENDS Naturals\nBar == 2\n====",
    "---- MODULE Derived ----\nEXTENDS Base\nBaz == Bar + 1\n====",
];

fn fixture(name: &str) -> (String, String, String) {
    let cases: Vec<Value> = serde_json::from_str(include_str!("finite-support.json")).unwrap();
    let case = cases
        .iter()
        .find(|case| case["name"] == name)
        .unwrap_or_else(|| panic!("{name} is in the finite support fixture"));
    (
        name.to_string(),
        case["source"].as_str().unwrap().to_string(),
        case["config"].as_str().unwrap().to_string(),
    )
}

fn rows() -> Vec<(String, String, String)> {
    let own = |row: (&str, &str, &str)| (row.0.to_string(), row.1.to_string(), row.2.to_string());
    vec![
        own(COUNTER),
        own(SHARED_OPERATORS),
        fixture("model-untyped"),
        fixture("temporal-fair"),
    ]
}

fn bundles() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("empty bundle", vec![]),
        ("one unrelated module", vec![UNUSED]),
        (
            "two unrelated modules, one extending the other",
            UNUSED_CHAIN.to_vec(),
        ),
    ]
}

fn parse(source: &str, config: &str, bundle: &[&str]) -> (Module, Config, Vec<Module>) {
    (
        rtla::parse_module(source).unwrap(),
        rtla::parse_config(config).unwrap(),
        bundle
            .iter()
            .map(|source| rtla::parse_module(source).unwrap())
            .collect(),
    )
}

fn serialized(label: &str, program: Result<rtlc::SafetyProgram, CompileError>) -> Value {
    serde_json::to_value(program.unwrap_or_else(|error| panic!("{label}: {error}"))).unwrap()
}

#[test]
fn v2_bundle_route_emits_the_ir_of_the_flat_route_when_no_linking_is_needed() {
    let mut mismatches = vec![];
    for (name, source, config) in rows() {
        for (bundle_name, bundle) in bundles() {
            let (module, config, modules) = parse(&source, &config, &bundle);
            let flat = serialized(&name, compile_with_options(&module, &config, V2));
            assert_eq!(flat["version"], 2, "{name}");
            match compile_modules_with_options(&module, &config, &modules, V2) {
                Ok(bundled) if serde_json::to_value(&bundled).unwrap() == flat => {}
                Ok(_) => mismatches.push(format!("{name} with {bundle_name}: different IR")),
                Err(error) => mismatches.push(format!("{name} with {bundle_name}: {error}")),
            }
        }
    }
    assert_eq!(mismatches, Vec::<String>::new());
}

#[test]
fn v1_bundle_route_still_compiles_every_row_that_needs_no_linking() {
    for (name, source, config) in rows() {
        for (bundle_name, bundle) in bundles() {
            let (module, config, modules) = parse(&source, &config, &bundle);
            let bundled = serialized(
                &format!("{name} through compile_modules_with_options with {bundle_name}"),
                compile_modules_with_options(&module, &config, &modules, V1),
            );
            assert_eq!(bundled["version"], 1, "{name} with {bundle_name}");
        }
    }
}

#[test]
fn v2_bundle_route_repeats_every_v1_refusal_for_inputs_that_need_no_linking() {
    let missing_identifier = "---- MODULE Counter ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x' = x + 1\nSafe == y < 3\n====";
    let reserved = "---- MODULE Naturals ----\nFoo == 1\n====";
    let refusals: [(&str, &str, Vec<&str>); 4] = [
        ("duplicate bundle module", COUNTER.1, vec![UNUSED, UNUSED]),
        ("reserved bundle module name", COUNTER.1, vec![reserved]),
        (
            "unresolved identifier, empty bundle",
            missing_identifier,
            vec![],
        ),
        (
            "unresolved identifier, unrelated module",
            missing_identifier,
            vec![UNUSED],
        ),
    ];
    for (name, source, bundle) in refusals {
        let (module, config, modules) = parse(source, COUNTER.2, &bundle);
        let v1 = compile_modules_with_options(&module, &config, &modules, V1)
            .err()
            .unwrap_or_else(|| panic!("{name}: IR v1 compiled the input"))
            .to_string();
        let v2 = compile_modules_with_options(&module, &config, &modules, V2)
            .err()
            .unwrap_or_else(|| panic!("{name}: IR v2 compiled the input"))
            .to_string();
        assert_ne!(v1, V2_LINKING_REFUSAL, "{name}");
        assert_eq!(v2, v1, "{name}");
    }
}
