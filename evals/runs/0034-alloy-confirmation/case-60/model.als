sig Node { edge: set Node }
fact Assumptions { ((Node -> Node) in ((iden . (none -> none)) . (Node -> Node))) }
Query: run { (((Node -> Node) in ((iden & edge) + (^edge))) and ((one (^(~(none -> none)))) or (some (~edge)))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
