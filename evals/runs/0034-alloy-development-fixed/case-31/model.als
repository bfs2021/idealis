sig Node { edge: set Node }
fact Assumptions { (not ((Node -> Node) in (^(edge . edge)))) }
Query: run { (lone (Node -> Node)) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
