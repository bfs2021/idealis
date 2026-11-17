---- MODULE Differential ----
Add(a, b) == 0
VARIABLE x
Init == x = Add(1,2)
Next == UNCHANGED x
Safe == x = 3
====