sig Node { edge: set Node }
fact Assumptions { no iden & ^edge }
assert Property { all n: Node | n not in n.^edge }
Query: check Property for 2 but 0 Int
Feasible: run {} for 2 but 0 Int
