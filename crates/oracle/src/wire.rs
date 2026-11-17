use crate::wire_observation::WireObservation;
use crate::{Case, CaseReport, CheckReport, Error, Observation, OracleGroup};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    version: u32,
    group: String,
    reference: String,
    implementations: Vec<String>,
    cases: Vec<Input>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    id: String,
    input: Value,
    reference: WireObservation,
    actual: Vec<WireObservation>,
}

#[derive(Serialize)]
struct Checked<'a> {
    kind: &'static str,
    report: CheckReport<&'a Value, &'a Value>,
}

/// Evaluate an observed JSON batch with the same semantics as `OracleGroup::check`.
/// Callbacks and clocks remain on the host; this boundary does not time implementations.
pub fn dispatch(bytes: &[u8]) -> Value {
    execute(bytes, |report| {
        serde_json::to_value(Checked {
            kind: "checked",
            report,
        })
        .expect("serializable oracle response")
    })
    .unwrap_or_else(|error| json!({"kind":"error","message":error.to_string()}))
}

/// Encode the complete response for byte-oriented hosts without changing the JSON contract.
pub fn dispatch_json(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    dispatch_into(bytes, &mut output);
    output
}

pub(crate) fn dispatch_into(bytes: &[u8], output: &mut Vec<u8>) {
    output.clear();
    if let Err(error) = execute(bytes, |report| {
        serde_json::to_writer(
            &mut *output,
            &Checked {
                kind: "checked",
                report,
            },
        )
        .expect("serialize oracle response");
    }) {
        output.clear();
        serde_json::to_writer(output, &json!({"kind":"error","message":error.to_string()}))
            .expect("serialize oracle error");
    }
}

fn borrow(observation: &Observation<Value>) -> Observation<&Value> {
    match observation {
        Observation::Value { value } => Observation::Value { value },
        Observation::Allowed { values } => Observation::Allowed {
            values: values.iter().collect(),
        },
        Observation::Unknown { reason } => Observation::Unknown {
            reason: reason.clone(),
        },
        Observation::Invalid { reason } => Observation::Invalid {
            reason: reason.clone(),
        },
        Observation::Error { message } => Observation::Error {
            message: message.clone(),
        },
    }
}

fn execute<R>(
    bytes: &[u8],
    emit: impl FnOnce(CheckReport<&Value, &Value>) -> R,
) -> Result<R, Error> {
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(Error("JSON batch exceeds 16 MiB".into()));
    }
    let request: Request =
        serde_json::from_slice(bytes).map_err(|error| Error(error.to_string()))?;
    if request.version != 1
        || request.cases.len() > 4096
        || request.implementations.len() > 256
        || request
            .cases
            .iter()
            .any(|case| case.actual.len() != request.implementations.len())
    {
        return Err(Error(
            "invalid version, batch dimensions or observation count".into(),
        ));
    }
    let mut group = OracleGroup::new(request.group);
    for (index, case) in request.cases.iter().enumerate() {
        group.case(&case.id, index);
    }
    group.reference(request.reference, |index| {
        borrow(&request.cases[*index].reference.0)
    });
    for (implementation, id) in request.implementations.iter().enumerate() {
        let cases = &request.cases;
        group.test_function(id, move |index| {
            borrow(&cases[*index].actual[implementation].0)
        });
    }
    let report = group.check()?;
    Ok(emit(CheckReport {
        version: report.version,
        group: report.group,
        reference: report.reference,
        implementations: report.implementations,
        status: report.status,
        cases: report
            .cases
            .into_iter()
            .map(|case| CaseReport {
                case: Case {
                    id: case.case.id,
                    input: &request.cases[case.case.input].input,
                },
                reference: case.reference,
                comparisons: case.comparisons,
            })
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::dispatch_into;
    use serde_json::Value;

    #[test]
    fn reused_output_replaces_success_and_error_without_trailing_bytes() {
        let request = br#"{"version":1,"group":"reused","reference":"oracle","implementations":["target"],"cases":[{"id":"one","input":null,"reference":{"kind":"value","value":7},"actual":[{"kind":"value","value":7}]}]}"#;
        let mut output = Vec::new();
        for input in [request.as_slice(), b"invalid", request.as_slice()] {
            dispatch_into(input, &mut output);
            let response: Value = serde_json::from_slice(&output).unwrap();
            assert_eq!(response, super::dispatch(input));
            assert_eq!(
                response["kind"],
                if input == b"invalid" {
                    "error"
                } else {
                    "checked"
                }
            );
        }
    }
}
