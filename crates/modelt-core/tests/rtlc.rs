use modelt_core::{Command, Engine, Message, Request, SafetyBounds};
use serde_json::json;

#[test]
fn legacy_api_uses_the_standalone_types_and_the_same_checker() {
    let program: rtlc::SafetyProgram = serde_json::from_value(json!({
        "format":"modelt.safety-program","version":1,"name":"OneState","variables":["x"],
        "initial":[{"name":"Init","steps":[{"kind":"assign","slot":0,"value":{"op":"literal","value":{"type":"int","value":"0"}}}]}],
        "next":[],"invariants":[],"assumptions":[],"checkDeadlock":false
    })).unwrap();
    let program: modelt_core::SafetyProgram = program;
    let bounds: rtlc::SafetyBounds = SafetyBounds {
        max_depth: 10,
        max_states: 100,
        max_work: 1000,
        max_temporal_nodes: None,
    };
    let expected: rtlc::SafetyResult = program.check(bounds).unwrap();
    assert_eq!((expected.states, expected.generated), (1, 1));
    let Message::Safety { result } = Engine::default().handle(Command {
        version: 1,
        request: Request::CheckSafety {
            program: program.clone(),
            bounds,
        },
    }) else {
        panic!("expected safety result")
    };
    assert_eq!(result, expected);
    let value: modelt_core::Value = rtlc::Value::Int("0".into());
    assert_eq!(value.normalize().unwrap(), rtlc::Value::Int("0".into()));
    let mut invalid = program;
    invalid.version = 0;
    let error: modelt_core::CoreError = invalid.check(bounds).unwrap_err();
    let error: rtlc::CoreError = error;
    let Message::Error { code, message } = Engine::default().handle(Command {
        version: 1,
        request: Request::CheckSafety {
            program: invalid,
            bounds,
        },
    }) else {
        panic!("expected error")
    };
    assert_eq!(code, error.code);
    assert_eq!(message, error.message);
}
