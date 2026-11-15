sig Node { edge: set Node }
fact Assumptions { ((no ((iden - (Node -> Node)) & ((Node -> Node) . edge))) iff (one (~edge))) }
Query: run { ((Node -> Node) = ((^iden) + (iden - (Node -> Node)))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
