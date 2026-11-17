---- MODULE Differential ----
EXTENDS Values
VARIABLE x
Hidden == 2
Init == x = Value
Next == UNCHANGED x
Safe == x = 1
====