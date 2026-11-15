sig Node { edge: set Node }
fact Assumptions { ((no iden) iff (one (none -> none))) }
assert Property { (some (*(edge . (none -> none)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
