sig Node { edge: set Node }
fact Assumptions { ((^(*edge)) in edge) }
assert Property { (some ((edge & edge) . (^iden))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
