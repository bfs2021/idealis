one sig A { value: Int }
fact Assumptions {  }
Query: run { some value } for 0
Feasible: run {} for 0
Pinned: run { (some value) and (some w_A_0: A | (A = w_A_0) and (value = ((w_A_0 -> -8))) and no none) } for 0
