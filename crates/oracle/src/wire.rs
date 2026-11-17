use crate::{Case, CaseReport, CheckReport, Error, Observation, OracleGroup};
use serde::Deserialize;
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
    reference: Observation<Value>,
    actual: Vec<Observation<Value>>,
}

/// Evaluate an observed JSON batch with the same semantics as `OracleGroup::check`.
/// Callbacks and clocks remain on the host; this boundary does not time implementations.
pub fn dispatch(bytes: &[u8]) -> Value {
    match execute(bytes) {
        Ok(report) => json!({"kind":"checked","report":report}),
        Err(error) => json!({"kind":"error","message":error.to_string()}),
    }
}

fn execute(bytes: &[u8]) -> Result<CheckReport<Value, Value>, Error> {
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
        request.cases[*index].reference.clone()
    });
    for (implementation, id) in request.implementations.iter().enumerate() {
        let cases = &request.cases;
        group.test_function(id, move |index| {
            cases[*index].actual[implementation].clone()
        });
    }
    let report = group.check()?;
    Ok(CheckReport {
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
                    input: request.cases[case.case.input].input.clone(),
                },
                reference: case.reference,
                comparisons: case.comparisons,
            })
            .collect(),
    })
}
