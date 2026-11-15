sig Node { edge: set Node }
fact Assumptions { (((*(iden - iden)) = iden) and (no (*(Node -> Node)))) }
assert Property { (one (Node -> Node)) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
