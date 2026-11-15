sig Node { edge: set Node }
fact Assumptions { ((one (edge . (*(none -> none)))) or (((^iden) + (Node -> Node)) = ((*(none -> none)) . edge))) }
Query: run { (lone (*(iden & edge))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
