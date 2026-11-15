sig Node { edge: set Node }
fact Assumptions { ((no (~(~(none -> none)))) and ((iden . iden) = ((iden . iden) + (*(none -> none))))) }
assert Property { (iden in (~iden)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
