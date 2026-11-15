sig Node { edge: set Node }
fact Assumptions { (((*(none -> none)) + iden) in (~(none -> none))) }
assert Property { ((Node -> Node) = iden) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
