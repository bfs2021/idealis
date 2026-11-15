sig Node { edge: set Node }
fact Assumptions { (((none -> none) in iden) and (no ((~(Node -> Node)) . (*(none -> none))))) }
assert Property { (some (((Node -> Node) - edge) - (Node -> Node))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
