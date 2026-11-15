sig Node { edge: set Node }
fact Assumptions { ((some (Node -> Node)) iff ((~(*iden)) = (((none -> none) + iden) . (none -> none)))) }
assert Property { (lone (^edge)) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
