#![cfg(not(target_arch = "wasm32"))]

use modelt_core::{SafetyBounds, SafetyProgram};
use serde_json::{Value, json};

fn integer(value: i32) -> Value {
    json!({"op":"literal","value":{"type":"int","value":value.to_string()}})
}

fn grid() -> Value {
    let read = |slot| json!({"op":"read","slot":slot,"next":false});
    json!({
        "format":"modelt.safety-program","version":1,"name":"Grid","variables":["a","b"],
        "initial":[{"name":"Init","steps":[
            {"kind":"assign","slot":0,"value":integer(0)},
            {"kind":"assign","slot":1,"value":integer(0)}
        ]}],
        "next":(0..2).map(|slot| json!({"name":format!("Step{slot}"),"steps":[
            {"kind":"assign","slot":slot,"value":{"op":"if",
                "condition":{"op":"lt","left":read(slot),"right":integer(3)},
                "consequent":{"op":"add","left":read(slot),"right":integer(1)},
                "alternative":integer(0)}},
            {"kind":"assign","slot":1-slot,"value":read(1-slot)}
        ]})).collect::<Vec<_>>(),
        "invariants":[{"name":"Safe","expression":{"op":"le","left":read(0),"right":integer(3)}}],
        "assumptions":[],"checkDeadlock":true
    })
}

fn bounds() -> SafetyBounds {
    SafetyBounds {
        max_depth: 100,
        max_states: 100,
        max_work: 100_000,
        max_temporal_nodes: None,
    }
}

#[test]
fn parallel_diamond_deduplicates_and_keeps_bfs_counts() {
    let program: SafetyProgram = serde_json::from_value(grid()).unwrap();
    let expected = program.check(bounds()).unwrap();
    assert_eq!(expected.states, 16);
    assert_eq!(expected.generated, 33);
    for workers in [1, 2, 4, 8] {
        let actual = program.check_parallel(bounds(), workers).unwrap();
        assert_eq!(actual.status, expected.status);
        assert_eq!(actual.states, expected.states);
        assert_eq!(actual.generated, expected.generated);
        assert_eq!(actual.depth, expected.depth);
        assert_eq!(actual.violation, expected.violation);
        assert!(actual.work <= bounds().max_work);
    }
}

#[test]
fn parallel_trace_is_the_same_shortest_trace() {
    let mut input = grid();
    input["invariants"][0]["expression"]["right"] = integer(1);
    let program: SafetyProgram = serde_json::from_value(input).unwrap();
    let expected = program.check(bounds()).unwrap();
    assert_eq!(expected.violation.as_ref().unwrap().trace.len(), 3);
    for workers in [2, 4, 8] {
        for _ in 0..10 {
            let actual = program.check_parallel(bounds(), workers).unwrap();
            assert_eq!(actual.violation, expected.violation);
            assert_eq!(actual.states, expected.states);
            assert_eq!(actual.generated, expected.generated);
        }
    }
}

#[test]
fn global_cuts_remain_bounded_and_worker_zero_is_rejected() {
    let program: SafetyProgram = serde_json::from_value(grid()).unwrap();
    assert!(program.check_parallel(bounds(), 0).is_err());
    for workers in [2, 4, 8] {
        let exact = program
            .check_parallel(
                SafetyBounds {
                    max_work: program.check(bounds()).unwrap().work,
                    ..bounds()
                },
                workers,
            )
            .unwrap();
        assert_eq!(exact.status, "complete");
        assert_eq!(exact.states, 16);
        assert_eq!(exact.generated, 33);
        for max_work in 1..program.check(bounds()).unwrap().work {
            let actual = program
                .check_parallel(
                    SafetyBounds {
                        max_work,
                        ..bounds()
                    },
                    workers,
                )
                .unwrap();
            assert_eq!(
                actual.status, "bounded",
                "workers={workers}, maxWork={max_work}, result={actual:?}"
            );
            assert_eq!(actual.reason.as_deref(), Some("work"));
            assert!(actual.work <= max_work);
        }
        for limited in [
            SafetyBounds {
                max_states: 3,
                ..bounds()
            },
            SafetyBounds {
                max_depth: 1,
                ..bounds()
            },
        ] {
            let actual = program.check_parallel(limited, workers).unwrap();
            let expected = program.check(limited).unwrap();
            assert_eq!(actual.status, expected.status);
            assert_eq!(actual.reason, expected.reason);
            assert_eq!(actual.states, expected.states);
            assert_eq!(actual.generated, expected.generated);
        }
    }
}

#[test]
fn converging_workers_validate_each_canonical_state_once() {
    let mut input = grid();
    for action in input["next"].as_array_mut().unwrap() {
        action["steps"][0]["value"]["condition"]["right"] = integer(31);
    }
    input["invariants"][0]["expression"] = json!({"op":"and", "left":{"op":"le", "left":{"op":"read","slot":0,"next":false},"right":integer(31)}, "right":{"op":"forall","bounds":[{"slot":0,"domain":{"kind":"range","lower":integer(0),"upper":integer(9)}}],"body":{"op":"lt","left":{"op":"bound","slot":0},"right":integer(10)}}});
    let program: SafetyProgram = serde_json::from_value(input).unwrap();
    let limits = SafetyBounds {
        max_states: 2000,
        max_work: 1_000_000,
        ..bounds()
    };
    let expected = program.check(limits).unwrap();
    assert_eq!(expected.states, 1024);
    for workers in [2, 4, 8] {
        let actual = program.check_parallel(limits, workers).unwrap();
        assert_eq!(actual.status, "complete");
        assert_eq!(actual.states, expected.states);
        assert_eq!(actual.generated, expected.generated);
        assert_eq!(
            actual.work, expected.work,
            "invariants must be checked once per canonical state, workers={workers}"
        );
    }
}
