sig Node { edge: set Node }
fact Assumptions { ((((Node -> Node) + (Node -> Node)) & (Node -> Node)) in (^(~(Node -> Node)))) }
Query: run { ((not (((none -> none) . iden) in (none -> none))) or ((some (((Node -> Node) + (Node -> Node)) . (Node -> Node))) iff (one ((Node -> Node) . (Node -> Node))))) } for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
