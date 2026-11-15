sig Node { edge: set Node }
fact Assumptions { ((some (^iden)) or (one ((edge - edge) & (*edge)))) }
assert Property { (((no ((edge . (Node -> Node)) + (^edge))) and (some (none -> none))) or ((no ((~edge) & edge)) implies ((^((Node -> Node) + edge)) = (Node -> Node)))) }
Query: check Property for 2 but exactly 2 Node, 0 Int
Feasible: run {} for 2 but exactly 2 Node, 0 Int
