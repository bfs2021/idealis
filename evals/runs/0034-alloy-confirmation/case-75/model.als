sig Node { edge: set Node }
fact Assumptions { ((no (~((Node -> Node) . iden))) and ((*(~iden)) = (^(*iden)))) }
assert Property { (some edge) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
