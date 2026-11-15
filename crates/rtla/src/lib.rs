pub mod ast;
mod config;
mod error;
mod lexer;
mod parser;
mod protocol;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use config::parse_config;
pub use error::{ErrorKind, ParseError};
pub use parser::{parse_expression, parse_module};
pub use protocol::{Request, Response, dispatch};

const MAX_NESTING: usize = 64;
