sig Node { edge: set Node }
fact Assumptions { ((no (Node -> Node)) implies (some ((~(Node -> Node)) + (*(none -> none))))) }
assert Property { ((edge & (iden + iden)) = (~((none -> none) & (Node -> Node)))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
Pinned: run { (not (((edge & (iden + iden)) = (~((none -> none) & (Node -> Node)))))) and (some w_Node_0: Node, w_Node_1: Node, w_Node_2: Node | (Node = w_Node_0 + w_Node_1 + w_Node_2) and (edge = ((w_Node_2 -> w_Node_2))) and w_Node_0 != w_Node_1 and w_Node_0 != w_Node_2 and w_Node_1 != w_Node_2) } for 3 but exactly 3 Node, 0 Int
