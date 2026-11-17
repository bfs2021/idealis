---- MODULE Differential ----
VARIABLE x
Hidden == 2
I == INSTANCE Values
Init == x = I!Value
Next == UNCHANGED x
Safe == x = 1
====