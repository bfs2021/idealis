sig A {} sig B {}
fact Assumptions {  }
Query: run { some A and no B } for exactly 1 A, 0 B, 0 Int
Feasible: run {} for exactly 1 A, 0 B, 0 Int
Pinned: run { (some A and no B) and (some w_A_0: A | (A = w_A_0) and (B = none) and no none) } for exactly 1 A, 0 B, 0 Int
