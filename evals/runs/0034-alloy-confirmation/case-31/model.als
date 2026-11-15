sig Node { edge: set Node }
fact Assumptions { ((lone (((Node -> Node) - (Node -> Node)) + iden)) or (lone ((^edge) - (*edge)))) }
assert Property { (no ((iden . iden) + (Node -> Node))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { (not ((no ((iden . iden) + (Node -> Node))))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
