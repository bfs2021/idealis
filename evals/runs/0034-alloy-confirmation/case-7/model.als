sig Owner { resource: Resource } sig Resource {}
fact Assumptions { all disj a,b: Owner | no a.resource & b.resource }
Query: run {  } for 3 but exactly 3 Owner, exactly 2 Resource, 0 Int
Feasible: run {} for 3 but exactly 3 Owner, exactly 2 Resource, 0 Int
