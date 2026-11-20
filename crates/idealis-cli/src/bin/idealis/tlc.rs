use super::{
    Result,
    modules::{Bundle, from_cwd, with_suffix},
};
use rtlc::{SafetyBounds, SafetyOptions};
use std::path::Path;

pub fn run(mut args: impl Iterator<Item = String>) -> Result<u8> {
    let mut spec = None;
    let mut config = None;
    let mut workers = 1;
    let mut no_deadlock = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-help" => {
                println!("Usage: idealis tlc [-config file] [-workers N] [-deadlock] spec");
                return Ok(1);
            }
            "-deadlock" => no_deadlock = true,
            "-config" => {
                config = Some(
                    args.next()
                        .filter(|s| !s.starts_with('-'))
                        .ok_or((1, "-config requires a file".into()))?,
                )
            }
            "-workers" => {
                let value = args.next().ok_or((1, "-workers requires a count".into()))?;
                workers = value
                    .parse()
                    .ok()
                    .filter(|n| (1..=256).contains(n))
                    .ok_or((1, format!("Unsupported worker count: {value}")))?;
                if cfg!(target_arch = "wasm32") && workers != 1 {
                    return Err((1, "The npm Wasm build supports -workers 1".into()));
                }
            }
            flag if flag.starts_with('-') => {
                return Err((1, format!("Unsupported TLC option: {flag}")));
            }
            _ if spec.is_some() => return Err((1, "Expected one root module".into())),
            _ => spec = Some(from_cwd(&with_suffix(&arg, ".tla"))),
        }
    }
    let path = spec.ok_or((1, "Usage: idealis tlc [options] spec".into()))?;

    let config_path = match config {
        None => path.with_extension("cfg"),
        Some(file) => {
            let cfg = with_suffix(&file, ".cfg");
            if from_cwd(&cfg).is_file() {
                from_cwd(&cfg)
            } else {
                path.parent().unwrap_or(Path::new(".")).join(cfg)
            }
        }
    };
    let text = std::fs::read_to_string(&config_path)
        .map_err(|e| (255, format!("{}: {e}", config_path.display())))?;
    let mut config =
        rtla::parse_config(&text).map_err(|e| (151, format!("{}: {e}", config_path.display())))?;
    if no_deadlock {
        config.check_deadlock = false;
    }
    let bundle = Bundle::load(&path, false).map_err(|msg| (150, msg))?;
    // Check the source before lowering: the executable IR omits proof declarations.
    rtla_compiler::analyze_modules(&bundle.root, &bundle.dependencies, true)
        .map_err(|e| (150, e.to_string()))?;
    let program = rtla_compiler::compile_modules(&bundle.root, &config, &bundle.dependencies)
        .map_err(|e| (150, e.to_string()))?;
    let result = program
        .check_with_options(
            SafetyBounds {
                max_depth: 1000,
                max_states: 100000,
                max_work: 10000000,
                max_temporal_nodes: Some(100000),
            },
            SafetyOptions {
                workers,
                ..Default::default()
            },
        )
        .map_err(|e| (255, e.to_string()))?;
    let code = match result.status.as_str() {
        "complete" => {
            println!("Model checking completed. No error has been found.");
            0
        }
        "assumption-violation" => 10,
        "deadlock" => 11,
        "invariant-violation" => 12,
        "property-violation" | "liveness-violation" => 13,
        _ => {
            return Err((
                255,
                format!(
                    "Model checking incomplete: {} ({})",
                    result.status,
                    result.reason.as_deref().unwrap_or("no result")
                ),
            ));
        }
    };
    if let Some(violation) = result.violation {
        match result.status.as_str() {
            "invariant-violation" => println!("Error: Invariant {} is violated.", violation.name),
            "deadlock" => println!("Error: Deadlock reached."),
            _ => println!("Error: {}: {}", result.status, violation.name),
        }
        println!(
            "{}",
            serde_json::to_string(&violation).expect("serializable counterexample")
        );
    }
    println!("{} distinct states found.", result.states);
    Ok(code)
}
