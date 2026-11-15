use crate::{Error, Problem};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    version: u32,
    request: Request,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Request {
    Solve {
        source: String,
        #[serde(rename = "maxWork")]
        max_work: u32,
    },
    Prepare {
        source: String,
    },
    Check {
        handle: u32,
        #[serde(rename = "maxWork")]
        max_work: u32,
    },
    Release {
        handle: u32,
    },
}
#[derive(Default)]
pub struct Engine {
    next: u32,
    problems: BTreeMap<u32, Problem>,
}
impl Engine {
    pub fn dispatch(&mut self, bytes: &[u8]) -> Value {
        match self.execute(bytes) {
            Ok(value) => value,
            Err(error) => json!({"kind":"error","code":error.code,"message":error.message}),
        }
    }
    fn execute(&mut self, bytes: &[u8]) -> Result<Value, Error> {
        let command: Command = serde_json::from_slice(bytes)
            .map_err(|error| Error::new("protocol", error.to_string()))?;
        if command.version != 1 {
            return Err(Error::new("protocol", "unsupported version"));
        }
        match command.request {
            Request::Solve { source, max_work } => {
                Ok(json!({"kind":"solution","result":Problem::parse(&source)?.solve(max_work)}))
            }
            Request::Prepare { source } => {
                let problem = Problem::parse(&source)?;
                self.next = self
                    .next
                    .checked_add(1)
                    .ok_or_else(|| Error::new("resource", "handles exhausted"))?;
                self.problems.insert(self.next, problem);
                Ok(json!({"kind":"prepared","handle":self.next}))
            }
            Request::Check { handle, max_work } => {
                let problem = self
                    .problems
                    .get(&handle)
                    .ok_or_else(|| Error::new("protocol", "unknown handle"))?;
                Ok(json!({"kind":"solution","result":problem.solve(max_work)}))
            }
            Request::Release { handle } => {
                if self.problems.remove(&handle).is_none() {
                    return Err(Error::new("protocol", "unknown handle"));
                }
                Ok(json!({"kind":"released"}))
            }
        }
    }
}
