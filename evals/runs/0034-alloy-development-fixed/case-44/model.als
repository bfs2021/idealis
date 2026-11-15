sig Node { edge: set Node }
fact Assumptions { ((((none -> none) - ((Node -> Node) + edge)) = (edge & (iden - (Node -> Node)))) or (((~iden) - (none -> none)) = ((~edge) + edge))) }
assert Property { (iden = (^(*edge))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
Pinned: run { (not ((iden = (^(*edge))))) and (some w_Node_0: Node, w_Node_1: Node, w_Node_2: Node | (Node = w_Node_0 + w_Node_1 + w_Node_2) and (edge = ((w_Node_2 -> w_Node_0))) and w_Node_0 != w_Node_1 and w_Node_0 != w_Node_2 and w_Node_1 != w_Node_2) } for 3 but exactly 3 Node, 0 Int
