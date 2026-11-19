use idealis_core::Engine;
use serde_json::json;

#[test]
fn source_is_compiled_and_checked_without_a_host_frontend() {
    let source = "---- MODULE Native ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x' = x + 1\nSafe == x < 3\n====";
    let config = "INIT Init\nNEXT Next\nINVARIANT Safe";
    let mut engine = Engine::default();
    for kind in ["compile-tla", "check-tla"] {
        let mut command =
            json!({"version":1,"request":{"kind":kind,"source":source,"config":config}});
        if kind == "check-tla" {
            command["request"]["bounds"] = json!({"maxDepth":10,"maxStates":100,"maxWork":10000});
        }
        let result =
            serde_json::to_value(engine.dispatch(&serde_json::to_vec(&command).unwrap())).unwrap();
        if kind == "compile-tla" {
            assert_eq!(result["kind"], "compiled");
            assert_eq!(result["program"]["variables"], json!(["x"]));
        } else {
            assert_eq!(result["kind"], "safety");
            assert_eq!(result["result"]["status"], "invariant-violation");
        }
    }
    let invalid =
        json!({"version":1,"request":{"kind":"compile-tla","source":"broken","config":config}});
    assert_eq!(
        serde_json::to_value(engine.dispatch(&serde_json::to_vec(&invalid).unwrap())).unwrap()["kind"],
        "error"
    );
}

#[test]
fn source_bundles_are_isolated_between_requests() {
    let source = "---- MODULE Root ----\nEXTENDS Counter\nSafe == x < 2\n====";
    let config = "INIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE";
    let mut engine = Engine::default();
    for (next, status) in [
        ("x' = x + 1", "invariant-violation"),
        ("UNCHANGED x", "complete"),
    ] {
        let dependency = format!(
            "---- MODULE Counter ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == {next}\n===="
        );
        let command = json!({"version":1,"request":{"kind":"check-tla","source":source,"config":config,"modules":{"Counter":dependency},"bounds":{"maxDepth":10,"maxStates":100,"maxWork":100000}}});
        let response =
            serde_json::to_value(engine.dispatch(&serde_json::to_vec(&command).unwrap())).unwrap();
        assert_eq!(response["result"]["status"], status, "{response}");
    }
    for modules in [json!({}), json!({"Counter":"---- MODULE Wrong ----\n===="})] {
        let command = json!({"version":1,"request":{"kind":"compile-tla","source":source,"config":config,"modules":modules}});
        assert_eq!(
            serde_json::to_value(engine.dispatch(&serde_json::to_vec(&command).unwrap())).unwrap()
                ["kind"],
            "error"
        );
    }
}

#[test]
fn source_ir_version_is_explicit_and_does_not_change_the_envelope() {
    let source = "---- MODULE Options ----\nVARIABLE x\nInit == x = 0\nNext == UNCHANGED x\n====";
    let config = "INIT Init\nNEXT Next";
    let mut engine = Engine::default();
    for kind in ["compile-tla", "check-tla"] {
        let mut command =
            json!({"version":1,"request":{"kind":kind,"source":source,"config":config}});
        if kind == "check-tla" {
            command["request"]["bounds"] =
                json!({"maxDepth":64,"maxStates":10000,"maxWork":1000000});
        }
        let default =
            serde_json::to_vec(&engine.dispatch(&serde_json::to_vec(&command).unwrap())).unwrap();
        command["request"]["irVersion"] = json!(1);
        let explicit =
            serde_json::to_vec(&engine.dispatch(&serde_json::to_vec(&command).unwrap())).unwrap();
        assert_eq!(default, explicit);
        command["request"]["irVersion"] = json!(2);
        let response =
            serde_json::to_value(engine.dispatch(&serde_json::to_vec(&command).unwrap())).unwrap();
        if kind == "compile-tla" {
            assert_eq!(response["kind"], "compiled");
            assert_eq!(response["program"]["version"], 2);
            assert_eq!(response["program"]["definitions"], json!([]));
        } else {
            assert_eq!(response["kind"], "safety");
            assert_eq!(response["result"]["status"], "complete");
        }
        for invalid in [
            json!(0),
            json!(3),
            json!(-1),
            json!(1.5),
            json!("2"),
            json!(null),
        ] {
            command["request"]["irVersion"] = invalid;
            let response =
                serde_json::to_value(engine.dispatch(&serde_json::to_vec(&command).unwrap()))
                    .unwrap();
            assert_eq!(response["kind"], "error");
            assert_eq!(response["code"], "protocol");
        }
    }
    let default = idealis_core::compile_tla(source, config).unwrap();
    let explicit = idealis_core::compile_tla_with_options(
        source,
        config,
        idealis_core::CompileOptions {
            irVersion: idealis_core::IrVersion::V1,
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(&default).unwrap(),
        serde_json::to_vec(&explicit).unwrap()
    );
    assert_eq!(
        idealis_core::compile_tla_with_options(
            source,
            config,
            idealis_core::CompileOptions {
                irVersion: idealis_core::IrVersion::V2
            }
        )
        .unwrap()
        .version,
        2
    );
}
