sig Node { edge: set Node }
fact Assumptions { (((*(*(Node -> Node))) in (none -> none)) and (one edge)) }
Query: run { (not (some ((*iden) - edge))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
