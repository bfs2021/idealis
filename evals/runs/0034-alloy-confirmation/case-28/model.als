sig Node { edge: set Node }
fact Assumptions { (no (~((none -> none) + (Node -> Node)))) }
Query: run { (some ((iden + edge) + (Node -> Node))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
