sig Node { edge: set Node }
fact Assumptions { (lone (none -> none)) }
Query: run { (((Node -> Node) + (edge + edge)) = edge) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { ((((Node -> Node) + (edge + edge)) = edge)) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (edge = ((w_Node_0 -> w_Node_0) + (w_Node_0 -> w_Node_1) + (w_Node_1 -> w_Node_0) + (w_Node_1 -> w_Node_1))) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
