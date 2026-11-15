sig Node { edge: set Node }
fact Assumptions { ((no (~(edge . (none -> none)))) and (some ((^edge) - ((none -> none) + (Node -> Node))))) }
Query: run { ((some ((none -> none) . (^(Node -> Node)))) or (some (^(^(none -> none))))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
