one sig N { value: Int }
fact Assumptions {  }
Query: run { some value } for 0
Feasible: run {} for 0

fact Observed { some w_N_0: N | (N = w_N_0) and (value = ((w_N_0 -> -8))) and no none }
