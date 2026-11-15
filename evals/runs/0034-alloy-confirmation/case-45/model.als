sig Node { edge: set Node }
fact Assumptions { ((some (((none -> none) - iden) & ((none -> none) . iden))) or ((edge - iden) in (^((none -> none) & iden)))) }
Query: run { (((*iden) . ((none -> none) + (Node -> Node))) = (*(iden + (Node -> Node)))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { ((((*iden) . ((none -> none) + (Node -> Node))) = (*(iden + (Node -> Node))))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
