---- MODULE Differential ----
VARIABLE x
I == INSTANCE Helper
Init == x = I!Value + 1
Next == UNCHANGED x
Safe == x = 2
====