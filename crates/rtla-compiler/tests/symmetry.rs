use rtlc::{SafetyBounds, SafetyProgram};

fn bounds() -> SafetyBounds {
    SafetyBounds {
        max_depth: 100,
        max_states: 10000,
        max_work: 10_000_000,
        max_temporal_nodes: None,
    }
}

fn input(count: usize, maximum: i32) -> (String, String) {
    let source = format!(
        r#"---- MODULE SymmetricCounters ----
EXTENDS Naturals, TLC
CONSTANT Nodes
VARIABLE counters
Symm == Permutations(Nodes)
Init == counters = [node \in Nodes |-> 0]
Next == \E node \in Nodes: counters' = [counters EXCEPT ![node] = IF @ < {maximum} THEN @ + 1 ELSE 0]
Safe == counters \in [Nodes -> 0..{maximum}]
===="#
    );
    let names = (0..count)
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    (
        source,
        format!("CONSTANT Nodes={{{names}}}\nINIT Init\nNEXT Next\nINVARIANT Safe\nSYMMETRY Symm"),
    )
}

fn compile(source: &str, config: &str) -> Result<SafetyProgram, rtla_compiler::CompileError> {
    rtla_compiler::compile(
        &rtla::parse_module(source).unwrap(),
        &rtla::parse_config(config).unwrap(),
    )
}

#[test]
fn full_node_permutations_reduce_counter_orbits_and_keep_plain_wire_unchanged() {
    let (source, config) = input(3, 2);
    let reduced = compile(&source, &config).unwrap();
    let wire = serde_json::to_value(&reduced).unwrap();
    assert_eq!(
        wire["symmetry"]["modelValues"],
        serde_json::json!(["p0", "p1", "p2"])
    );
    let plain = compile(&source, &config.replace("\nSYMMETRY Symm", "")).unwrap();
    assert!(
        serde_json::to_value(&plain)
            .unwrap()
            .get("symmetry")
            .is_none()
    );
    let original = plain.check(bounds()).unwrap();
    assert_eq!(
        (
            original.status.as_str(),
            original.states,
            original.generated
        ),
        ("complete", 27, 82)
    );
    let actual = reduced.check(bounds()).unwrap();
    assert_eq!(
        (
            actual.status.as_str(),
            actual.states,
            actual.generated,
            actual.depth
        ),
        ("complete", 10, 31, 6)
    );
    for workers in [1, 2, 8] {
        assert_eq!(reduced.check_parallel(bounds(), workers).unwrap(), actual);
    }
}

#[test]
fn multiple_functions_preserve_same_node_correlations_and_global_scalars() {
    let source = r#"---- MODULE Joint ----
EXTENDS TLC
CONSTANT Nodes
VARIABLES first, second, phase
Symm == Permutations(Nodes)
Init == /\ first = [n \in Nodes |-> FALSE] /\ second = [n \in Nodes |-> FALSE] /\ phase = FALSE
Next == \/ (\E n \in Nodes: /\ first' = [first EXCEPT ![n] = ~@] /\ UNCHANGED <<second, phase>>)
        \/ (\E n \in Nodes: /\ second' = [second EXCEPT ![n] = ~@] /\ UNCHANGED <<first, phase>>)
        \/ (/\ phase' = ~phase /\ UNCHANGED <<first, second>>)
Safe == /\ first \in [Nodes -> BOOLEAN] /\ second \in [Nodes -> BOOLEAN] /\ phase \in BOOLEAN
===="#;
    let config = "CONSTANT Nodes={p0,p1,p2}\nINIT Init\nNEXT Next\nINVARIANT Safe\nSYMMETRY Symm";
    let reduced = compile(source, config).unwrap().check(bounds()).unwrap();
    let plain = compile(source, &config.replace("\nSYMMETRY Symm", ""))
        .unwrap()
        .check(bounds())
        .unwrap();
    assert_eq!((plain.states, plain.generated), (128, 897));
    assert_eq!(
        (reduced.status.as_str(), reduced.states, reduced.generated),
        ("complete", 40, 281)
    );
}

#[test]
fn symmetry_keeps_concrete_shortest_counterexamples_and_budget_cuts() {
    let (source, config) = input(3, 2);
    let source = source.replace(
        "Safe == counters \\in [Nodes -> 0..2]",
        "Safe == \\A node \\in Nodes: counters[node] < 2",
    );
    let reduced = compile(&source, &config).unwrap();
    let result = reduced.check(bounds()).unwrap();
    assert_eq!(result.status, "invariant-violation");
    let trace = &result.violation.as_ref().unwrap().trace;
    assert_eq!(trace.len(), 3);
    assert_eq!(
        trace[0].values[0],
        rtlc::Value::Map(vec![
            (
                rtlc::Value::ModelValue("p0".into()),
                rtlc::Value::Int("0".into())
            ),
            (
                rtlc::Value::ModelValue("p1".into()),
                rtlc::Value::Int("0".into())
            ),
            (
                rtlc::Value::ModelValue("p2".into()),
                rtlc::Value::Int("0".into())
            ),
        ])
    );
    for max_work in [1, 2, 5, 17, 50] {
        let result = reduced
            .check(SafetyBounds {
                max_work,
                ..bounds()
            })
            .unwrap();
        assert_eq!(result.status, "bounded");
        assert_eq!(result.reason.as_deref(), Some("work"));
        assert!(result.work <= max_work);
    }
    for limits in [
        SafetyBounds {
            max_states: 1,
            ..bounds()
        },
        SafetyBounds {
            max_depth: 0,
            ..bounds()
        },
    ] {
        let result = reduced.check(limits).unwrap();
        assert_eq!(result.status, "bounded");
    }
    assert!(reduced.check_with_coverage(bounds()).is_err());
}

#[test]
fn unsupported_symmetry_conditions_are_rejected_instead_of_ignored() {
    let (source, config) = input(3, 2);
    for altered in [
        config.replace("{p0, p1, p2}", "{p0}"),
        config.replace("{p0, p1, p2}", "{p0,p0}"),
        config.replace("{p0, p1, p2}", "{n_a,n_b}"),
        config.replace("{p0, p1, p2}", "{1,2}"),
        config.replace("SYMMETRY Symm", "SYMMETRY Missing"),
    ] {
        assert!(compile(&source, &altered).is_err(), "{altered}");
    }
    let reduced = compile(&source, &config).unwrap();
    let wire = serde_json::to_value(&reduced).unwrap();
    for (field, value) in [
        (
            "properties",
            serde_json::json!([{"name":"P","formula":{"kind":"always","formula":{"kind":"atom","expression":{"op":"literal","value":{"type":"bool","value":true}}}}}]),
        ),
        (
            "constraints",
            serde_json::json!([{"name":"C","expression":{"op":"literal","value":{"type":"bool","value":true}}}]),
        ),
        (
            "initialProperties",
            serde_json::json!([{"name":"P","expression":{"op":"literal","value":{"type":"bool","value":true}}}]),
        ),
    ] {
        let mut input = wire.clone();
        input[field] = value;
        assert!(
            serde_json::from_value::<SafetyProgram>(input)
                .unwrap()
                .check(bounds())
                .is_err()
        );
    }
    let named = source
        .replace(
            "Safe == counters \\in [Nodes -> 0..2]",
            "Safe == counters[p0] = 0",
        )
        .replace("CONSTANT Nodes", "CONSTANTS Nodes, p0");
    assert!(
        compile(&named, &format!("CONSTANT p0=p0\n{config}"))
            .unwrap()
            .check(bounds())
            .is_err()
    );
}

#[test]
fn symmetry_does_not_treat_shadowed_permutations_as_the_tlc_builtin() {
    let (source, config) = input(3, 2);
    let constant = source.replace("CONSTANT Nodes", "CONSTANTS Nodes, Permutations");
    assert_eq!(
        compile(&constant, &format!("CONSTANT Permutations=0\n{config}"))
            .unwrap_err()
            .message,
        "SYMMETRY requires TLC.Permutations"
    );
    let variable = source
        .replace("VARIABLE counters", "VARIABLES counters, Permutations")
        .replace(
            "Init == counters = [node \\in Nodes |-> 0]",
            "Init == counters = [node \\in Nodes |-> 0] /\\ Permutations = 0",
        )
        .replace("\nSafe ==", " /\\ UNCHANGED Permutations\nSafe ==");
    assert_eq!(
        compile(&variable, &config).unwrap_err().message,
        "SYMMETRY requires TLC.Permutations"
    );
}
