sig Node { edge: set Node }
fact Assumptions { ((no (edge - (none -> none))) or (one iden)) }
assert Property { ((edge in iden) implies (lone edge)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
