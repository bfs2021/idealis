use rtlc::ir::{Expression, Scalar, Step};
use rtlc::{SafetyBounds, SafetyProgram};

fn integer(value: i32) -> Expression {
    Expression::Literal {
        value: Scalar::Int(value),
    }
}
fn read() -> Expression {
    Expression::Read {
        slot: 0,
        next: false,
    }
}
#[path = "../examples/counter.rs"]
mod counter;
use counter::ring;

fn bounds() -> SafetyBounds {
    SafetyBounds {
        max_depth: 10,
        max_states: 100,
        max_work: 1000,
        max_temporal_nodes: None,
    }
}

#[test]
fn typed_ir_checks_without_idealis() {
    let result = ring().check(bounds()).unwrap();
    assert_eq!(result.status, "complete");
    assert_eq!((result.states, result.generated), (4, 5));
    for limits in [
        SafetyBounds {
            max_states: 2,
            ..bounds()
        },
        SafetyBounds {
            max_depth: 1,
            ..bounds()
        },
        SafetyBounds {
            max_work: 1,
            ..bounds()
        },
    ] {
        let result = ring().check(limits).unwrap();
        assert_eq!(result.status, "bounded");
        assert!(result.work <= limits.max_work);
    }
}
#[test]
fn shortest_violation_and_empty_initial_states_remain_distinct() {
    let mut program = ring();
    program.invariants[0].expression = Expression::Le {
        left: Box::new(read()),
        right: Box::new(integer(1)),
    };
    let result = program.check(bounds()).unwrap();
    assert_eq!(result.status, "invariant-violation");
    assert_eq!(
        result
            .violation
            .unwrap()
            .trace
            .iter()
            .map(|step| step.values.clone())
            .collect::<Vec<_>>(),
        vec![
            vec![rtlc::Value::Int("0".into())],
            vec![rtlc::Value::Int("1".into())],
            vec![rtlc::Value::Int("2".into())]
        ]
    );
    program.initial.clear();
    assert_eq!(program.check(bounds()).unwrap().status, "empty");
    program.invariants[0].expression = Expression::Literal {
        value: Scalar::Bool(false),
    };
    assert_eq!(program.check(bounds()).unwrap_err().code, "protocol");
}
#[test]
fn wire_contract_roundtrips_and_rejects_internal_references() {
    let encoded = serde_json::to_value(ring()).unwrap();
    assert_eq!(encoded["format"], "modelt.safety-program");
    assert_eq!(
        encoded["initial"][0]["steps"][0]["value"]["value"],
        serde_json::json!({"type":"int","value":"0"})
    );
    let decoded: SafetyProgram = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.check(bounds()).unwrap().states, 4);
    let mut invalid = ring();
    invalid.initial[0].steps[0] = Step::Assign {
        slot: 0,
        value: Expression::Literal {
            value: Scalar::Ref(0),
        },
    };
    assert!(invalid.check(bounds()).is_err());
}
