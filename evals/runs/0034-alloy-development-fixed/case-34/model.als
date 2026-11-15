sig Node { edge: set Node }
fact Assumptions { (no ((none -> none) - (none -> none))) }
Query: run { ((some ((iden - edge) + edge)) iff ((no (~(~iden))) and (some (^(edge . edge))))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
