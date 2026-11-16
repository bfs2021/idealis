---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Live == ~ (\E n \in 0..1: [] (x = n))
Init == x = 0
Next == x' = 1 - x
Spec == Init /\ [][Next]_x /\ WF_x(Next)
====