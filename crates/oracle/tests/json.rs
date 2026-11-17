use idealis_oracle::dispatch;
use serde_json::{Value, json};
fn request() -> Value {
    json!({"version":1,"group":"wire","reference":"oracle","implementations":["native","wasm"],"cases":[
        {"id":"one","input":{"source":"model"},"reference":{"kind":"value","value":7},"actual":[{"kind":"value","value":7},{"kind":"value","value":8}]}
    ]})
}
#[test]
fn json_batches_use_the_same_group_and_preserve_inputs_and_failures() {
    let reply = dispatch(&serde_json::to_vec(&request()).unwrap());
    assert_eq!(reply["kind"], "checked");
    assert_eq!(reply["report"]["status"], "mismatch");
    assert_eq!(
        reply["report"]["cases"][0]["case"]["input"],
        json!({"source":"model"})
    );
    assert_eq!(
        reply["report"]["cases"][0]["comparisons"][0]["status"],
        "matched"
    );
    let mut input = request();
    input["cases"][0]["reference"] = json!({"kind":"unknown","reason":"budget"});
    let reply = dispatch(&serde_json::to_vec(&input).unwrap());
    assert_eq!(reply["report"]["status"], "unknown");
}
#[test]
fn malformed_or_vacuous_batches_cannot_match() {
    for path in ["version", "cases", "actual", "extra", "duplicate"] {
        let mut input = request();
        match path {
            "version" => input["version"] = json!(2),
            "cases" => input["cases"] = json!([]),
            "actual" => input["cases"][0]["actual"] = json!([]),
            "extra" => input["typo"] = json!(true),
            "duplicate" => input["implementations"] = json!(["native", "native"]),
            _ => unreachable!(),
        }
        assert_eq!(
            dispatch(&serde_json::to_vec(&input).unwrap())["kind"],
            "error",
            "{path}"
        );
    }
    assert_eq!(dispatch(b"not json")["kind"], "error");
}
