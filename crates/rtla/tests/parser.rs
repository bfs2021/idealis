use rtla::ast::ExpressionKind;
use rtla::{ErrorKind, parse_config, parse_expression, parse_module};

#[test]
fn parses_upstream_channel_without_a_checker() {
    let module = parse_module(include_str!("fixtures/Channel.tla")).unwrap();
    assert_eq!(module.name, "Channel");
    assert_eq!(module.variables[0].name, "chan");
    assert_eq!(module.constants[0].name, "Data");
    assert_eq!(module.definitions.len(), 6);
    assert_eq!(module.theorems.len(), 1);
    assert_eq!(module.definitions[0].name, "TypeInvariant");
}

#[test]
fn preserves_nested_comments_strings_and_locations() {
    let expression = parse_expression("(* outer (* nested *) *)\nIF TRUE THEN \"a\\\"b\" ELSE \"c\"").unwrap();
    assert_eq!(expression.location.line, 2);
    assert_eq!(expression.location.column, 1);
    let ExpressionKind::If { consequent, .. } = expression.node else { panic!("expected IF") };
    assert!(matches!(consequent.node, ExpressionKind::String { ref value } if value == "a\"b"));
    for source in ["(* missing", "\"missing", "[a |-> 1", "1 = 2 = 3", "1 / 2 * 3", "TRUE /\\ FALSE \\/ TRUE"] {
        assert_eq!(parse_expression(source).unwrap_err().kind, ErrorKind::Syntax, "{source}");
    }
}

#[test]
fn parses_config_and_diagnoses_incomplete_input() {
    let config = parse_config("CONSTANT N = 3 S = {a, b}\nINIT Init\nNEXT Next\nINVARIANT Safe\nCHECK_DEADLOCK FALSE").unwrap();
    assert_eq!(config.init.as_deref(), Some("Init"));
    assert_eq!(config.constants.len(), 2);
    assert!(!config.check_deadlock);
    for source in ["INIT", "INIT Init\nINIT Other", "CONSTANT N =", "CHECK_DEADLOCK maybe"] {
        assert_eq!(parse_config(source).unwrap_err().kind, ErrorKind::Syntax);
    }
    let error = parse_module("---- MODULE M ----\nINSTANCE Other\n====").unwrap_err();
    assert_eq!(error.kind, ErrorKind::Unsupported);
    assert_eq!((error.location.line, error.location.column), (2, 1));
}

#[test]
fn parses_die_hard_junctions_and_keeps_the_bullets() {
    let module = parse_module(include_str!("fixtures/DieHard.tla")).unwrap();
    assert_eq!(module.name, "DieHard");
    let initial = module.definitions.iter().find(|definition| definition.name == "Init").unwrap();
    assert!(matches!(initial.body.node, ExpressionKind::And { ref operands, .. } if operands.len() == 2));
}
