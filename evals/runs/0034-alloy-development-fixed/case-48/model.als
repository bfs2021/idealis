sig Node { edge: set Node }
fact Assumptions { (no (^(~(Node -> Node)))) }
assert Property { (not ((lone (~(~(Node -> Node)))) or (one (^((none -> none) - (Node -> Node)))))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
