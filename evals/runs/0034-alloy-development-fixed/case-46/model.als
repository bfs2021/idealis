sig Node { edge: set Node }
fact Assumptions { ((((~(none -> none)) & iden) = ((Node -> Node) + ((Node -> Node) - iden))) iff (lone ((^iden) + ((none -> none) - (Node -> Node))))) }
Query: run { (some (~(none -> none))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
