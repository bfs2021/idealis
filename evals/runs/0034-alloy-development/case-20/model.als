sig N { edge: set N }
fact Assumptions {  }
assert Property { all n: N | (n <: edge) = (n -> n.edge) and (edge :> n) = ((edge.n) -> n) }
Query: check Property for 2 but 0 Int
Feasible: run {} for 2 but 0 Int
