lone sig A {}
fact Assumptions {  }
Query: run { some A } for 0 but 0 Int
Feasible: run {} for 0 but 0 Int
Pinned: run { (some A) and (some w_A_0: A | (A = w_A_0) and no none) } for 0 but 0 Int
