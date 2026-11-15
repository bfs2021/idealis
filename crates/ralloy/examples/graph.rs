use ralloy::{Model, Status};

fn main() -> Result<(), ralloy::Error> {
    let model = Model::parse(
        "sig Node { edge: set Node } fact { no iden & edge } \
         assert Acyclic { no iden & ^edge } \
         check Acyclic for 2 but exactly 2 Node, 0 Int",
    )?;
    let result = model.analyze(0, 2_000_000)?;
    assert_eq!(result.status, Status::Counterexample);
    let instance = result.instance.unwrap();
    assert_eq!(instance.signatures["Node"].len(), 2);
    assert_eq!(instance.fields["edge"].len(), 2);
    assert_eq!(model.analyze(0, 0)?.status, Status::Unknown);
    assert_eq!(
        Model::parse("sig A {} fact { some A no A } assert P { no A } check P for 1 but 0 Int")?
            .analyze(0, 100_000)?
            .status,
        Status::NoInstance
    );
    assert!(Model::parse("var sig A {} run {}").is_err());
    println!("standalone ralloy consumer checked");
    Ok(())
}
