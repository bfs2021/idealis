sig Node { edge: set Node }
fact Assumptions { ((iden in ((edge & (Node -> Node)) + (edge - edge))) implies ((*(Node -> Node)) in edge)) }
assert Property { (((some (~(~(Node -> Node)))) or (some (iden & (~edge)))) or (not (some (none -> none)))) }
Query: check Property for 3 but exactly 3 Node, 0 Int
Feasible: run {} for 3 but exactly 3 Node, 0 Int
