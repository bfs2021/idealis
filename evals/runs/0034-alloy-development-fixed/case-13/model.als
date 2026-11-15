sig A {}
fact Assumptions {  }
Query: run { all a: A | a in A } for 0 but 0 Int
Feasible: run {} for 0 but 0 Int
Pinned: run { (all a: A | a in A) and ((A = none)) } for 0 but 0 Int
