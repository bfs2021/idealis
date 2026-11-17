---- MODULE Differential ----
EXTENDS Numbers
VARIABLE x
Replacement == 4
Init == x = Value
Next == UNCHANGED x
Safe == x = 5
====