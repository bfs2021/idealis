sig Node { edge: set Node }
fact Assumptions { (one (^(none -> none))) }
assert Property { ((((Node -> Node) in edge) iff (((iden . edge) & (^edge)) in ((edge . (none -> none)) & ((Node -> Node) + (Node -> Node))))) or ((((iden & (none -> none)) + (~edge)) in (~(*edge))) iff (some (((none -> none) + edge) & iden)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
