sig N { edge: set N }
fact Assumptions {  }
Query: run { some edge } for 0 but 0 Int
Feasible: run {} for 0 but 0 Int

fact Observed { (N = none) and (no edge) }
