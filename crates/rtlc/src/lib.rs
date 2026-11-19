mod bounded_drop;
mod expansions;
pub use expansions::{
    ExpansionOptions, ExpansionStats, IncrementalSafetyResult, SafetyExpansionCache,
};
pub mod ir;
mod safety;
#[cfg(test)]
mod safety_definition_tests;
mod safety_definitions;
mod safety_expressions;
mod safety_values;
#[cfg(not(target_arch = "wasm32"))]
mod safety_work;
mod temporal;
mod value;

pub use ir::{SafetyBounds, SafetyProgram, SafetyResult};
pub use value::{CoreError, Value};
