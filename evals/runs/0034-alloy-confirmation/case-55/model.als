sig Node { edge: set Node }
fact Assumptions { (one (Node -> Node)) }
assert Property { (((Node -> Node) & ((none -> none) - (none -> none))) = ((~(none -> none)) . iden)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
