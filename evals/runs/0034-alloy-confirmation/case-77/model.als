sig Node { edge: set Node }
fact Assumptions { (not (some ((Node -> Node) & ((Node -> Node) . iden)))) }
Query: run { (lone (~(^iden))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
