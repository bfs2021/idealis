use rtlc::Value;

fn int(text: &str) -> Value {
    Value::Int(text.into())
}

fn bytes(text: &str) -> Value {
    Value::Bytes(text.into())
}

fn model(text: &str) -> Value {
    Value::ModelValue(text.into())
}

fn field(name: &str, value: Value) -> (String, Value) {
    (name.into(), value)
}

fn assert_rejected(value: Value, message: &str) {
    let error = value.normalize().expect_err("the value is rejected");
    assert_eq!((error.code, error.message.as_str()), ("value", message));
}

#[test]
fn record_reports_a_field_value_error_before_a_duplicate_name() {
    assert_rejected(
        Value::Record(vec![field("a", Value::Null), field("a", int("01"))]),
        "Noncanonical integer",
    );
}

#[test]
fn record_reports_a_duplicate_name_before_a_later_field_value_error() {
    assert_rejected(
        Value::Record(vec![
            field("a", Value::Null),
            field("a", Value::Null),
            field("b", int("01")),
        ]),
        "Duplicate record field",
    );
}

#[test]
fn record_reports_the_first_failing_field_in_input_order() {
    assert_rejected(
        Value::Record(vec![field("b", int("01")), field("a", bytes("F"))]),
        "Noncanonical integer",
    );
}

#[test]
fn map_reports_a_key_error_before_its_value_error() {
    assert_rejected(
        Value::Map(vec![(model("1x"), int("01"))]),
        "Invalid model value name",
    );
}

#[test]
fn map_reports_a_key_error_before_a_duplicate_key() {
    assert_rejected(
        Value::Map(vec![
            (int("1"), Value::Null),
            (int("01"), Value::Null),
            (int("1"), Value::Null),
        ]),
        "Noncanonical integer",
    );
}

#[test]
fn map_reports_a_value_error_before_a_duplicate_key() {
    assert_rejected(
        Value::Map(vec![(int("1"), Value::Null), (int("1"), bytes("F"))]),
        "Invalid lowercase hex bytes",
    );
}

#[test]
fn map_reports_a_duplicate_key_before_a_later_entry_error() {
    assert_rejected(
        Value::Map(vec![
            (int("1"), Value::Null),
            (int("1"), Value::Null),
            (int("2"), int("01")),
        ]),
        "Duplicate map key",
    );
}

#[test]
fn list_reports_its_first_failing_element() {
    assert_rejected(
        Value::List(vec![int("1"), bytes("F"), int("01")]),
        "Invalid lowercase hex bytes",
    );
}

#[test]
fn list_reports_an_error_nested_in_an_earlier_element() {
    assert_rejected(
        Value::List(vec![Value::List(vec![model("")]), int("01")]),
        "Invalid model value name",
    );
}

#[test]
fn set_reports_its_first_failing_element() {
    assert_rejected(
        Value::Set(vec![int("1"), model(""), int("01")]),
        "Invalid model value name",
    );
}

#[test]
fn set_reports_an_element_error_after_an_equal_earlier_element() {
    assert_rejected(
        Value::Set(vec![int("2"), int("2"), bytes("F")]),
        "Invalid lowercase hex bytes",
    );
}
