sig Node { edge: set Node }
fact Assumptions { (((*(none -> none)) & (none -> none)) in (~(*(none -> none)))) }
assert Property { (not ((((none -> none) . edge) + ((none -> none) - iden)) = (edge & (^edge)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
Pinned: run { (not ((not ((((none -> none) . edge) + ((none -> none) - iden)) = (edge & (^edge)))))) and (some w_Node_0: Node, w_Node_1: Node | (Node = w_Node_0 + w_Node_1) and (no edge) and w_Node_0 != w_Node_1) } for 2 but exactly 2 Node, 0 Int
