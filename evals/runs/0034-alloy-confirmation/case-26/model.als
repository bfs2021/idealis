sig Node { edge: set Node }
fact Assumptions { (((^iden) in (*((none -> none) . (none -> none)))) and (some (none -> none))) }
assert Property { (no (^iden)) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
