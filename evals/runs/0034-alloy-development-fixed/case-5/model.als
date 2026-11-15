sig Owner { resource: some Resource } sig Resource {}
fact Assumptions {  }
Query: run { no Owner } for 0 but 0 Int
Feasible: run {} for 0 but 0 Int
Pinned: run { (no Owner) and ((Owner = none) and (Resource = none) and (no resource)) } for 0 but 0 Int
