sig Node { edge: set Node }
fact Assumptions { (one (Node -> Node)) }
Query: run { (some (iden + (*(Node -> Node)))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
