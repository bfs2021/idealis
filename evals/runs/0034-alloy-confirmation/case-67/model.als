sig Node { edge: set Node }
fact Assumptions { (one (iden + (Node -> Node))) }
assert Property { (no (iden . iden)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
