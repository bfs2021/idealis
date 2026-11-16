---- MODULE Differential ----
EXTENDS Integers
VARIABLE x
Init == x = 0
Next == x' = x + 1
Limit == x < 2
Live == <> (x = 2)
Spec == Init /\ [][Next]_x /\ WF_x(Next)
====