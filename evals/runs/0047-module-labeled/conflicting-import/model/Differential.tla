---- MODULE Differential ----
EXTENDS Left, Right
VARIABLE x
Init == x = Value
Next == UNCHANGED x
Safe == TRUE
====