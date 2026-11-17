---- MODULE Differential ----
EXTENDS Left
VARIABLE x
Init == x = 0
Next == UNCHANGED x
Safe == TRUE
====