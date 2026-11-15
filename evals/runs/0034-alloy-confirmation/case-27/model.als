sig Node { edge: set Node }
fact Assumptions { (not (lone (none -> none))) }
assert Property { ((some iden) or (no (^edge))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
