---- MODULE Differential ----
CONSTANT Add(_, _)
VARIABLE x
Init == x = Add(1,2)
Next == UNCHANGED x
Safe == x = 3
====