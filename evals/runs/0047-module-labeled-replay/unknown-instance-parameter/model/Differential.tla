---- MODULE Differential ----
VARIABLE x
I == INSTANCE Values WITH Missing <- 1
Init == x = I!Value
Next == UNCHANGED x
Safe == TRUE
====