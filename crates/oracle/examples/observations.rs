use serde_json::json;
fn main() {
    let request = json!({"version":1,"group":"observations","reference":"model","implementations":["sut"],"cases":[{"id":"one","input":{"operation":"read"},"reference":{"kind":"value","value":7},"actual":[{"kind":"value","value":7}]}]});
    let report = idealis_oracle::dispatch(&serde_json::to_vec(&request).unwrap());
    let encoded = idealis_oracle::dispatch_json(&serde_json::to_vec(&request).unwrap());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&encoded).unwrap(),
        report
    );
    assert_eq!(report["kind"], "checked");
    assert_eq!(report["report"]["status"], "matched");
    assert_eq!(idealis_oracle::dispatch(b"invalid")["kind"], "error");
    println!("standalone oracle JSON consumer checked");
}
