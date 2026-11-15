sig Node { edge: set Node }
fact Assumptions { ((none -> none) = (none -> none)) }
assert Property { (((none -> none) in (Node -> Node)) implies ((no (*(none -> none))) implies (no (((Node -> Node) . (Node -> Node)) . (iden + (none -> none)))))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
