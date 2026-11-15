sig Node { edge: set Node }
fact Assumptions { (not ((^(none -> none)) in ((*iden) . (edge . edge)))) }
assert Property { ((some (^(none -> none))) and (iden in edge)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
