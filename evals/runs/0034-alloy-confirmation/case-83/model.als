sig Node { edge: set Node }
fact Assumptions { (lone edge) }
assert Property { (one iden) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { (not ((one iden))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
