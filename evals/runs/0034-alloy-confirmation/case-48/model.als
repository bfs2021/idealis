sig Node { edge: set Node }
fact Assumptions { (lone (Node -> Node)) }
Query: run { (((^iden) . (Node -> Node)) = (~(edge - edge))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
