sig Node { edge: set Node }
fact Assumptions { (one ((edge + (none -> none)) - edge)) }
assert Property { (((lone (*(^edge))) implies (((~(Node -> Node)) & edge) in iden)) or ((some edge) and (edge in (*(edge + edge))))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
