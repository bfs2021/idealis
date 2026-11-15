sig Node { edge: set Node }
fact Assumptions { (not (one ((^(Node -> Node)) . iden))) }
Query: run { (lone (iden & (edge & (Node -> Node)))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { ((lone (iden & (edge & (Node -> Node))))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
