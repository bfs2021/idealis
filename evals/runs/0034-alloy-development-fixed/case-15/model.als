sig A {}
fact Assumptions {  }
Query: run { one a,b: A | a = b } for 1 but exactly 1 A, 0 Int
Feasible: run {} for 1 but exactly 1 A, 0 Int
Pinned: run { (one a,b: A | a = b) and (some w_A_0: A | (A = w_A_0) and no none) } for 1 but exactly 1 A, 0 Int
