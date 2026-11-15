sig Node { edge: set Node }
fact Assumptions { no iden & edge }
assert Property { no iden & ^edge }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { (not (no iden & ^edge)) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (edge = ((w_Node_0 -> w_Node_1) + (w_Node_1 -> w_Node_0))) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
