sig Node { edge: set Node }
fact Assumptions { (lone (none -> none)) }
Query: run { (((none -> none) = (((none -> none) + (Node -> Node)) - (none -> none))) or ((no (iden . ((none -> none) & edge))) implies (no (Node -> Node)))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
