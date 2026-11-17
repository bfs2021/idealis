---- MODULE Differential ----
VARIABLE x
I == INSTANCE Wrapper
Init == x = I!J!Value
Next == UNCHANGED x
Safe == x = 3
====