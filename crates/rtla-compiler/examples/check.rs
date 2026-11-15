fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module = rtla::parse_module(
        "---- MODULE Counter ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x' = x + 1\nSafe == x < 3\n====",
    )?;
    let config = rtla::parse_config("INIT Init\nNEXT Next\nINVARIANT Safe")?;
    let program = rtla_compiler::compile(&module, &config)?;
    let result = program.check(rtlc::SafetyBounds {
        max_depth: 10,
        max_states: 100,
        max_work: 10000,
        max_temporal_nodes: None,
    })?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
