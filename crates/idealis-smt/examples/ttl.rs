use idealis_smt::Problem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let problem = Problem::parse(
        "(set-logic QF_IDL)(declare-const ttl Int)(declare-const elapsed Int)(assert (= ttl 7))(assert (= elapsed ttl))",
    )?;
    let result = problem.solve(100_000);
    assert_eq!(result.status, "sat");
    assert_eq!(result.values["ttl"], "7");
    assert_eq!(result.values["elapsed"], "7");

    let contradiction = Problem::parse(
        "(declare-const x Int)(declare-const y Int)(assert (< x y))(assert (<= y x))",
    )?;
    assert_eq!(contradiction.solve(100_000).status, "unsat");
    let bounded = problem.solve(0);
    assert_eq!(bounded.status, "unknown");
    assert_eq!(bounded.reason.as_deref(), Some("work"));
    assert!(bounded.values.is_empty());
    assert_eq!(
        Problem::parse("(declare-const x Real)").err().unwrap().code,
        "unsupported"
    );
    println!("standalone idealis-smt consumer checked");
    Ok(())
}
