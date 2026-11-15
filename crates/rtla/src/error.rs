use crate::ast::Location;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    Syntax,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Location,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} at {}:{}",
            self.message, self.location.line, self.location.column
        )
    }
}
impl std::error::Error for ParseError {}
