sig Node { edge: set Node }
fact Assumptions { (one (none -> none)) }
Query: run { ((((iden & edge) - (^edge)) in iden) and (some iden)) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
