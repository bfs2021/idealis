sig Node { edge: set Node }
fact Assumptions { (edge in iden) }
Query: run { ((one ((*(Node -> Node)) & ((Node -> Node) - edge))) or (((iden . (iden & iden)) = ((*iden) + (Node -> Node))) and (some (edge + ((none -> none) . (Node -> Node)))))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
