sig Node { edge: set Node }
fact Assumptions { ((Node -> Node) in (~(^iden))) }
Query: run { (((Node -> Node) & (edge + iden)) in (edge & (Node -> Node))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
