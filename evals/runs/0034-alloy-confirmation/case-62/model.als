sig Node { edge: set Node }
fact Assumptions { ((~(none -> none)) in (~(edge & (Node -> Node)))) }
assert Property { ((no (Node -> Node)) implies ((lone ((^(Node -> Node)) - (iden - iden))) and (lone (((Node -> Node) . (Node -> Node)) . (^edge))))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
