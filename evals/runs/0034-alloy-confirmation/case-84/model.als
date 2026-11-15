sig Node { edge: set Node }
fact Assumptions { ((one (((none -> none) - (Node -> Node)) - (none -> none))) and (some iden)) }
assert Property { (not ((^(edge . iden)) in (^iden))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
