sig Node { edge: set Node }
fact Assumptions { (some ((edge & edge) - (none -> none))) }
assert Property { ((some (*(none -> none))) iff (iden in edge)) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { (not (((some (*(none -> none))) iff (iden in edge)))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (edge = ((w_Node_1 -> w_Node_0))) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
