sig Node { edge: set Node }
fact Assumptions { ((^(*(Node -> Node))) in (~((Node -> Node) - iden))) }
assert Property { (no ((^(Node -> Node)) + (none -> none))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
