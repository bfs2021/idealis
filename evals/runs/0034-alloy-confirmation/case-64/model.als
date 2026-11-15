sig Node { edge: set Node }
fact Assumptions { (((*(none -> none)) = (*((none -> none) . edge))) or (some edge)) }
Query: run { ((~(^(Node -> Node))) in (none -> none)) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
