use rtlc::{Value, ir::SafetyViolation};

fn values(items: &[Value]) -> String {
    items.iter().map(render).collect::<Vec<_>>().join(", ")
}

fn record_name(name: &str) -> bool {
    name.chars().all(|c| c == '_' || c.is_alphanumeric())
        && name.chars().any(char::is_alphabetic)
        && !(name.len() > 3 && (name.starts_with("WF_") || name.starts_with("SF_")))
}

fn render(value: &Value) -> String {
    match value {
        Value::Bool(true) => "TRUE".into(),
        Value::Bool(false) => "FALSE".into(),
        Value::Int(n) | Value::ModelValue(n) => n.clone(),
        Value::String(s) => serde_json::to_string(s).expect("serializable string"),
        Value::Set(items) => format!("{{{}}}", values(items)),
        Value::List(items) => format!("<<{}>>", values(items)),
        Value::Record(fields) if !fields.iter().all(|(name, _)| record_name(name)) => {
            let entries = fields
                .iter()
                .map(|(name, v)| {
                    let key = serde_json::to_string(name).expect("serializable string key");
                    format!("{key} :> {}", render(v))
                })
                .collect::<Vec<_>>();
            format!("({})", entries.join(" @@ "))
        }
        Value::Record(fields) => {
            let entries = fields
                .iter()
                .map(|(name, v)| format!("{name} |-> {}", render(v)))
                .collect::<Vec<_>>();
            format!("[{}]", entries.join(", "))
        }
        Value::Map(entries) => {
            let entries = entries
                .iter()
                .map(|(k, v)| format!("{} :> {}", render(k), render(v)))
                .collect::<Vec<_>>();
            format!("({})", entries.join(" @@ "))
        }
        // Portable-only values; the TLA evaluator never produces them.
        Value::Null | Value::Bytes(_) => serde_json::to_string(value).expect("serializable value"),
    }
}

pub fn print_diff(variables: &[String], violation: &SafetyViolation, status: &str) {
    let steps = &violation.trace;
    let prefix = if variables.len() == 1 { "" } else { "/\\ " };
    for (index, step) in steps.iter().enumerate() {
        let label = step.action.as_deref().unwrap_or("Initial predicate");
        if !(status == "invariant-violation" && steps.len() == 1) {
            println!("State {}: <{label}>", index + 1);
        }
        // TLCTrace prints the final safety state in full except for a successor
        // of the initial state. A lasso keeps predecessor-relative diffs.
        let full_final = violation.loop_start.is_none()
            && index + 1 == steps.len()
            && (status == "deadlock" || index > 1);
        for (slot, (name, value)) in variables.iter().zip(&step.values).enumerate() {
            if index == 0 || full_final || steps[index - 1].values[slot] != *value {
                println!("{prefix}{name} = {}", render(value));
            }
        }
        println!();
    }
    if let Some(start) = violation.loop_start {
        let action = violation.loop_action.as_deref().unwrap_or_default();
        if start + 1 == steps.len() {
            println!("State {}: Stuttering", steps.len() + 1);
        } else {
            println!("Back to state {}: {action}", start + 1);
        }
    }
}
