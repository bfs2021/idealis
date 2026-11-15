sig Node { edge: set Node }
fact Assumptions { ((some (((Node -> Node) + edge) + (iden + iden))) or (some (Node -> Node))) }
Query: run { ((*(none -> none)) = (edge + (~(Node -> Node)))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
