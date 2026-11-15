sig Node { edge: set Node }
fact Assumptions { ((one edge) implies (edge in (^((none -> none) + iden)))) }
assert Property { (some (^iden)) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
