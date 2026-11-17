use rtla::{parse_expression, parse_module};

#[test]
fn qualified_operators_and_instance_substitutions_are_preserved() {
    let source = "---- MODULE Root ----\nVARIABLE x\nA == INSTANCE Counter WITH value <- x, Limit <- 2\nLOCAL INSTANCE Helpers\nInit == A!Init\nNext == A!Next(1)\n====";
    let module = serde_json::to_value(parse_module(source).unwrap()).unwrap();
    assert_eq!(module["instances"][0]["name"], "A");
    assert_eq!(module["instances"][0]["module"], "Counter");
    assert_eq!(module["instances"][0]["substitutions"][0]["name"], "value");
    assert_eq!(module["instances"][1]["isLocal"], true);
    assert_eq!(module["definitions"][0]["body"]["name"], "A!Init");
    assert_eq!(module["definitions"][1]["body"]["operator"], "A!Next");
    let expression = serde_json::to_value(parse_expression("A!B!Apply(1)").unwrap()).unwrap();
    assert_eq!(expression["operator"], "A!B!Apply");
}

#[test]
fn unsupported_instance_forms_are_rejected() {
    for body in [
        "A(n) == INSTANCE Counter WITH Limit <- n",
        "A == INSTANCE Counter WITH Limit <- 1, Limit <- 2",
        "Init == LET A == INSTANCE Counter IN A!Init",
    ] {
        assert!(parse_module(&format!("---- MODULE Root ----\n{body}\n====")).is_err());
    }
}
