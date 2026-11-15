sig N { edge: set N }
fact Assumptions {  }
Query: run { edge = none + none and no ^(none & none) } for 1 but 0 Int
Feasible: run {} for 1 but 0 Int
Pinned: run { (edge = none + none and no ^(none & none)) and ((N = none) and (no edge)) } for 1 but 0 Int
