fn main() -> Result<(), rtla::ParseError> {
    let module = rtla::parse_module(include_str!("../tests/fixtures/Channel.tla"))?;
    println!("{}: {} definitions", module.name, module.definitions.len());
    Ok(())
}
