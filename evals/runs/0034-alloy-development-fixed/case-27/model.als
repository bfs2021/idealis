sig Node { edge: set Node }
fact Assumptions { (((edge - (Node -> Node)) . ((Node -> Node) + edge)) = (*iden)) }
Query: run { (no ((~(Node -> Node)) . (^edge))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
