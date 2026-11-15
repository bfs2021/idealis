sig Node { edge: set Node }
fact Assumptions { ((((Node -> Node) - iden) - (none -> none)) in (*iden)) }
assert Property { ((one (~(iden . (Node -> Node)))) implies ((some (~(*(none -> none)))) or (some (*((Node -> Node) . (none -> none)))))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
