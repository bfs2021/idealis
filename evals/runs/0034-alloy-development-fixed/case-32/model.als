sig Node { edge: set Node }
fact Assumptions { (not (some (*(*(Node -> Node))))) }
Query: run { ((one edge) and (no (none -> none))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
