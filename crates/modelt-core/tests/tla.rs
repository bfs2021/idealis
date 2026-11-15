use modelt_core::Engine;
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
