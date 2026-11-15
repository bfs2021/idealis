sig Node { edge: set Node }
fact Assumptions { ((one (none -> none)) or (((iden . edge) + (edge & edge)) in (~(Node -> Node)))) }
Query: run { (not ((some (^((none -> none) - iden))) and (no ((edge - iden) & ((Node -> Node) & (Node -> Node)))))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { ((not ((some (^((none -> none) - iden))) and (no ((edge - iden) & ((Node -> Node) & (Node -> Node))))))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
