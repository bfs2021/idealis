sig Node { edge: set Node }
fact Assumptions { (some ((none -> none) + iden)) }
assert Property { ((no ((Node -> Node) . (^edge))) and (one ((edge + iden) & (none -> none)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { (not (((no ((Node -> Node) . (^edge))) and (one ((edge + iden) & (none -> none)))))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
