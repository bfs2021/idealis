//! Differential tests and comparative benchmarks against an explicit oracle.
mod bench;
mod group;
mod report;

pub use bench::{BenchCase, BenchConfig, BenchFailure, BenchReport, Measurement, Statistics};
pub use group::OracleGroup;
pub use report::{Case, CaseReport, CheckReport, Comparison, Error, Observation, Status};
