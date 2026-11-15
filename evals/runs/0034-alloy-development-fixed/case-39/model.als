sig Node { edge: set Node }
fact Assumptions { (not (one (none -> none))) }
Query: run { ((not (some edge)) or ((one ((Node -> Node) . ((Node -> Node) . (none -> none)))) iff (((~edge) + ((none -> none) + edge)) = (*((Node -> Node) - iden))))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
Pinned: run { (((not (some edge)) or ((one ((Node -> Node) . ((Node -> Node) . (none -> none)))) iff (((~edge) + ((none -> none) + edge)) = (*((Node -> Node) - iden)))))) and (some w_Node_0: Node, w_Node_1: Node, w_Node_2: Node | (Node = w_Node_0 + w_Node_1 + w_Node_2) and (no edge) and w_Node_0 != w_Node_1 and w_Node_0 != w_Node_2 and w_Node_1 != w_Node_2) } for 3 but exactly 3 Node, 0 Int
