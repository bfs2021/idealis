sig Node { edge: set Node }
fact Assumptions { (not (no (~(edge + edge)))) }
Query: run { (some (^(^edge))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { ((some (^(^edge)))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (edge = ((w_Node_1 -> w_Node_0))) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
