sig Node { edge: set Node }
fact Assumptions { ((one (((Node -> Node) . (none -> none)) . (~(Node -> Node)))) and (((Node -> Node) + ((Node -> Node) & (Node -> Node))) = iden)) }
Query: run { ((no ((^(none -> none)) - ((Node -> Node) . iden))) and (((^(^(none -> none))) = iden) or (no (((Node -> Node) - (Node -> Node)) + edge)))) } for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
