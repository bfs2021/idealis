sig Node { edge: set Node }
fact Assumptions { ((no (*(~iden))) or (no (Node -> Node))) }
assert Property { ((((((none -> none) . iden) & (^iden)) in (edge - (*edge))) iff (no (^(^edge)))) iff (some (Node -> Node))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
