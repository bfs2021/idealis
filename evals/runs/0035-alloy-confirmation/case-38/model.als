sig N { edge: set N }
fact Assumptions {  }
Query: run { no iden & ^edge } for 2 but exactly 2 N, 0 Int
Feasible: run {} for 2 but exactly 2 N, 0 Int

fact Observed { some w_N_0: N, w_N_1: N | (N = w_N_0 + w_N_1) and (edge = ((w_N_0 -> w_N_0) + (w_N_0 -> w_N_1) + (w_N_1 -> w_N_0) + (w_N_1 -> w_N_1))) and w_N_0 != w_N_1 }
