sig Node { edge: set Node }
fact Assumptions { (one ((^(none -> none)) + iden)) }
assert Property { ((((~edge) & (iden - iden)) = edge) and ((*(none -> none)) = ((^(none -> none)) . (*iden)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
