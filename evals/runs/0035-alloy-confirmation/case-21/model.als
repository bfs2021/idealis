sig N { edge: set N }
fact Assumptions {  }
assert Property { no iden & ^edge }
Query: check Property for 3 but exactly 3 N, 0 Int
Feasible: run {} for 3 but exactly 3 N, 0 Int

fact Observed { some w_N_0: N, w_N_1: N, w_N_2: N | (N = w_N_0 + w_N_1 + w_N_2) and (edge = ((w_N_0 -> w_N_0) + (w_N_0 -> w_N_1) + (w_N_0 -> w_N_2) + (w_N_1 -> w_N_1) + (w_N_1 -> w_N_2) + (w_N_2 -> w_N_0) + (w_N_2 -> w_N_1) + (w_N_2 -> w_N_2))) and w_N_0 != w_N_1 and w_N_0 != w_N_2 and w_N_1 != w_N_2 }
