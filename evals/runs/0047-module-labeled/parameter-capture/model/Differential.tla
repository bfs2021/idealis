---- MODULE Differential ----
VARIABLE x
I == INSTANCE Params
Init == x = I!Value(7)
Next == UNCHANGED x
Safe == x = 7
====