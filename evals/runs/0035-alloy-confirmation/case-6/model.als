sig N { edge: some N }
fact Assumptions {  }
Query: run { no N } for 0 but 0 Int
Feasible: run {} for 0 but 0 Int

fact Observed { (N = none) and (no edge) }
