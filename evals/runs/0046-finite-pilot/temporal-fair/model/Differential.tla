---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Live == \A n \in 0..1: <> (x = n)
Spec == Init /\ [][Next]_x /\ WF_x(Next)
Init == x = 0
Next == x' = 1 - x
====