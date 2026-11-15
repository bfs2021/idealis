sig N { edge: set N }
fact Assumptions { edge = ~edge }
assert Property { edge.edge in N -> N }
Query: check Property for 2 but 0 Int
Feasible: run {} for 2 but 0 Int
