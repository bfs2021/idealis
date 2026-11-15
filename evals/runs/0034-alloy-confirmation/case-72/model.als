sig Node { edge: set Node }
fact Assumptions { (no (((Node -> Node) - edge) + (^iden))) }
assert Property { (((some ((Node -> Node) & (iden & (none -> none)))) or (some edge)) implies ((some (^((none -> none) - (Node -> Node)))) iff (((~(none -> none)) & edge) in (*(^iden))))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
