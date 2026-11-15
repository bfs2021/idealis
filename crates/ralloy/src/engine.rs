use crate::{Error, ErrorKind, Instance, Model};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    request: Request,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Request {
    Analyze {
        source: String,
        command: usize,
        #[serde(rename = "maxWork")]
        max_work: u64,
    },
    CheckInstance {
        source: String,
        command: usize,
        instance: Instance,
        #[serde(rename = "maxWork")]
        max_work: u64,
    },
}
#[derive(Default)]
pub struct Engine;
impl Engine {
    pub fn dispatch(&mut self, input: &[u8]) -> Value {
        match self.execute(input) {
            Ok(value) => value,
            Err(error) => json!({"kind":"error","error":error}),
        }
    }
    fn execute(&mut self, input: &[u8]) -> Result<Value, Error> {
        let envelope: Envelope = serde_json::from_slice(input)
            .map_err(|error| Error::new(ErrorKind::Protocol, error.to_string()))?;
        if envelope.version != 1 {
            return Err(Error::new(ErrorKind::Protocol, "unsupported version"));
        }
        match envelope.request {
            Request::Analyze {
                source,
                command,
                max_work,
            } => Ok(
                json!({"kind":"analysis","result":Model::parse(&source)?.analyze(command, max_work)?}),
            ),
            Request::CheckInstance {
                source,
                command,
                instance,
                max_work,
            } => Ok(
                json!({"kind":"instance-check","result":Model::parse(&source)?.check_instance(command, &instance, max_work)?}),
            ),
        }
    }
}
