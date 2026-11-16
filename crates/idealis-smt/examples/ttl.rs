use idealis_smt::{Assumption, Problem, Solver};

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
    let mut solver = Solver::parse("(declare-const x Int)(declare-const b Bool)")?;
    solver.push(1)?;
    solver.assert("(= (* 2 x) (ite b 6 8))")?;
    let assumptions = [Assumption {
        symbol: "b".into(),
        positive: true,
    }];
    assert_eq!(
        solver.check_assuming(&assumptions, 100_000)?.values["x"],
        "3"
    );
    solver.assert("(= x 4)")?;
    assert_eq!(
        solver.check_assuming(&assumptions, 100_000)?.status,
        "unsat"
    );
    assert_eq!(solver.check(100_000).values["x"], "4");
    solver.pop(1)?;
    solver.assert("(= x 9)")?;
    assert_eq!(solver.check(100_000).values["x"], "9");
    println!("standalone idealis-smt consumer checked");
    Ok(())
}
