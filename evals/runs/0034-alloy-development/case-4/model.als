one sig Owner { resource: lone Resource } sig Resource {}
fact Assumptions {  }
Query: run { no resource } for 0 but 0 Int
Feasible: run {} for 0 but 0 Int
Pinned: run { (no resource) and (some w_Owner_0: Owner | (Owner = w_Owner_0) and (Resource = none) and (no resource) and no none) } for 0 but 0 Int
