sig Node { edge: set Node }
fact Assumptions { (edge = (~(*iden))) }
assert Property { (not (not (no (((none -> none) + edge) - (Node -> Node))))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
