sig Node { edge: set Node }
fact Assumptions { ((lone iden) and (lone (^(*edge)))) }
assert Property { (lone (^((none -> none) . (none -> none)))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
