sig Node { edge: set Node }
fact Assumptions { (some (^(none -> none))) }
Query: run { (((one (Node -> Node)) and (((Node -> Node) + (iden - iden)) = iden)) and (some (iden - (edge . edge)))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
