sig Node { edge: set Node }
fact Assumptions { (one (Node -> Node)) }
Query: run { (((no ((none -> none) . ((Node -> Node) . (Node -> Node)))) iff (some (Node -> Node))) implies ((iden = (*(~(Node -> Node)))) and ((^(^(none -> none))) in (~(^edge))))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
