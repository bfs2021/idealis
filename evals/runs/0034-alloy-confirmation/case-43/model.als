sig Node { edge: set Node }
fact Assumptions { (((~(Node -> Node)) in (((none -> none) + (Node -> Node)) . (*edge))) and (((iden & (none -> none)) . (iden . iden)) in edge)) }
assert Property { ((((edge . iden) = ((iden & (Node -> Node)) & ((Node -> Node) . (Node -> Node)))) iff (no iden)) or (no (((none -> none) . (none -> none)) - (edge . (none -> none))))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
