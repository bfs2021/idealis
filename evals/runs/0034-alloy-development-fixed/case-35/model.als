sig Node { edge: set Node }
fact Assumptions { ((no ((^iden) - iden)) iff (iden = (((none -> none) . (Node -> Node)) + (none -> none)))) }
assert Property { (no (none -> none)) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
