sig N { edge: set N } pred Cyclic { some n: N | n in n.^edge }
fact Assumptions {  }
Query: run { Cyclic[] } for 2 but 0 Int
Feasible: run {} for 2 but 0 Int
Pinned: run { (Cyclic[]) and (some w_N_0: N | (N = w_N_0) and (edge = ((w_N_0 -> w_N_0))) and no none) } for 2 but 0 Int
