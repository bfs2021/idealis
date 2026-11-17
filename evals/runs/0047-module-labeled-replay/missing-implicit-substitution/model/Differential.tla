---- MODULE Differential ----
VARIABLE x
I == INSTANCE Values
Init == x = I!Value
Next == UNCHANGED x
Safe == TRUE
====