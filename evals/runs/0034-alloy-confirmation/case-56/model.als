sig Node { edge: set Node }
fact Assumptions { ((Node -> Node) = iden) }
assert Property { (some ((^edge) & edge)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
