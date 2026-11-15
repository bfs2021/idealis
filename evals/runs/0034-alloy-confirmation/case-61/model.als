sig Node { edge: set Node }
fact Assumptions { ((one (iden . (edge + iden))) or (((*(Node -> Node)) & (*(Node -> Node))) = (none -> none))) }
assert Property { (lone (*iden)) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
