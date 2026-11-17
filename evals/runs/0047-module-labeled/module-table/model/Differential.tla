---- MODULE Differential ----
EXTENDS Numbers
VARIABLE x
Init == x = Value
Next == UNCHANGED x
Safe == x = 7
====