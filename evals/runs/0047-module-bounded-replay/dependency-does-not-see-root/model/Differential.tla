---- MODULE Differential ----
VARIABLE x
Missing == 1
I == INSTANCE Helper
Init == x = I!Value
Next == UNCHANGED x
Safe == x = 1
====