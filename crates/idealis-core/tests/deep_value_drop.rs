use idealis_core::{
    Command, ConcurrentHistory, ConformanceResult, Edge, Effect, Engine, FiniteModel, FiniteState,
    HistoryEvent, HostEvent, HostMessage, HostTape, Initial, Message, Request, Response, State,
    Status, Step, Trace, Value,
};

const SHALLOW_DEPTH: usize = 1_000;
const SHALLOW_STACK_BYTES: usize = 8 * 1024 * 1024;
const DEEP_DEPTH: usize = 200_000;
const DEEP_STACK_BYTES: usize = 512 * 1024;

const UNSUPPORTED_VERSION: &str = "Unsupported protocol version";
const UNKNOWN_HANDLE: &str = "Unknown model handle";
const NO_ACTIVE_SESSION: &str = "No active session";

fn list_chain(depth: usize) -> Value {
    (0..depth).fold(Value::Bool(true), |inner, _| Value::List(vec![inner]))
}

fn trace(depth: usize) -> Trace {
    Trace {
        format: "modelt.portable-trace".into(),
        version: 1,
        model: "Deep".into(),
        initial: list_chain(depth),
        steps: vec![Step {
            action: list_chain(depth),
            observation: list_chain(depth),
        }],
    }
}

fn assert_refused(version: u32, request: Request, expected: &str) {
    let Message::Error { code, message } = Engine::default().handle(Command { version, request })
    else {
        panic!("the request is refused before it is evaluated")
    };
    assert_eq!(code, "protocol");
    assert_eq!(message, expected);
}

fn run_on_stack(stack_bytes: usize, depth: usize, work: fn(usize)) {
    std::thread::Builder::new()
        .stack_size(stack_bytes)
        .spawn(move || work(depth))
        .unwrap()
        .join()
        .expect("the value is built, refused and dropped without panicking");
}

macro_rules! drop_case {
    ($name:ident, $work:expr) => {
        mod $name {
            use super::*;

            #[test]
            fn shallow() {
                run_on_stack(SHALLOW_STACK_BYTES, SHALLOW_DEPTH, $work);
            }

            #[test]
            fn deep() {
                run_on_stack(DEEP_STACK_BYTES, DEEP_DEPTH, $work);
            }
        }
    };
}

drop_case!(refused_canonicalize_unsupported_version, |depth| {
    assert_refused(
        0,
        Request::Canonicalize {
            value: list_chain(depth),
        },
        UNSUPPORTED_VERSION,
    );
});

drop_case!(refused_start_loaded_unknown_handle, |depth| {
    assert_refused(
        1,
        Request::StartLoaded {
            handle: 7,
            actions: vec![list_chain(depth)],
        },
        UNKNOWN_HANDLE,
    );
});

drop_case!(refused_check_loaded_unknown_handle, |depth| {
    assert_refused(
        1,
        Request::CheckLoaded {
            handle: 7,
            traces: vec![trace(depth)],
        },
        UNKNOWN_HANDLE,
    );
});

drop_case!(refused_check_history_unknown_handle, |depth| {
    assert_refused(
        1,
        Request::CheckHistory {
            handle: 7,
            history: ConcurrentHistory {
                format: "modelt.concurrent-history".into(),
                version: 1,
                model: "Deep".into(),
                initial: list_chain(depth),
                events: vec![HistoryEvent::Invoke {
                    id: "call".into(),
                    action: list_chain(depth),
                }],
            },
            max_nodes: 10,
        },
        UNKNOWN_HANDLE,
    );
});

drop_case!(refused_load_model_unsupported_version, |depth| {
    assert_refused(
        0,
        Request::LoadModel {
            model: FiniteModel {
                format: "modelt.finite-model".into(),
                version: 1,
                name: "Deep".into(),
                initial: vec![Initial {
                    key: "start".into(),
                    observation: list_chain(depth),
                }],
                states: vec![FiniteState {
                    key: "start".into(),
                    violations: vec![],
                    transitions: vec![Edge {
                        action: list_chain(depth),
                        target: "start".into(),
                        observation: Value::Null,
                    }],
                }],
            },
        },
        UNSUPPORTED_VERSION,
    );
});

drop_case!(refused_replay_host_unsupported_version, |depth| {
    assert_refused(
        0,
        Request::ReplayHost {
            tape: HostTape {
                format: "modelt.host-tape".into(),
                version: 1,
                events: vec![HostEvent {
                    operation: "write".into(),
                    input: list_chain(depth),
                    output: Value::Null,
                }],
            },
        },
        UNSUPPORTED_VERSION,
    );
});

drop_case!(refused_resume_without_session, |depth| {
    assert_refused(
        1,
        Request::Resume {
            id: 1,
            response: Response::Observation {
                observation: list_chain(depth),
            },
        },
        NO_ACTIVE_SESSION,
    );
});

drop_case!(owned_message_value, |depth| {
    drop(Message::Value {
        value: list_chain(depth),
    });
});

drop_case!(owned_message_host, |depth| {
    drop(Message::Host {
        snapshot: list_chain(depth),
    });
});

drop_case!(owned_message_results, |depth| {
    drop(Message::Results {
        results: vec![ConformanceResult {
            status: Status::Matched,
            checked_steps: 1,
            actual: list_chain(depth),
            expected: vec![list_chain(depth)],
            trace: trace(depth),
        }],
    });
});

drop_case!(owned_message_effect, |depth| {
    drop(Message::Effect {
        effect: Effect::ModelStep {
            id: 1,
            states: vec![State {
                key: "start".into(),
                state: list_chain(depth),
            }],
            action: list_chain(depth),
        },
    });
});

drop_case!(owned_host_message_payload, |depth| {
    drop(HostMessage {
        id: 1,
        from: "a".into(),
        to: "b".into(),
        payload: list_chain(depth),
    });
});
