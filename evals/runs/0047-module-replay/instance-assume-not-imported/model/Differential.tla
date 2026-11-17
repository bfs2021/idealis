---- MODULE Differential ----
VARIABLE x
I == INSTANCE Values WITH N <- 1
Init == x = I!Value
Next == UNCHANGED x
Safe == x = 1
====