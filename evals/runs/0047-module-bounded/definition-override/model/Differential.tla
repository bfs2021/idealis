---- MODULE Differential ----
VARIABLE x
Value == 0
Replacement == 2
Init == x = Value
Next == UNCHANGED x
Safe == x = 2
====