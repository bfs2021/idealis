sig Node { edge: set Node }
fact Assumptions { (no (*(^(none -> none)))) }
Query: run { ((no (*((Node -> Node) & (none -> none)))) or (not (lone (~((Node -> Node) + iden))))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
