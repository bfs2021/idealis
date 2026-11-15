sig Node { edge: set Node }
fact Assumptions { ((^(Node -> Node)) = ((*(Node -> Node)) & (*iden))) }
assert Property { (not (lone (*(none -> none)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
