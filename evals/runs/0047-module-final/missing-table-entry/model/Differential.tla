---- MODULE Differential ----
F(a) == 0
VARIABLE x
Init == x = F(2)
Next == UNCHANGED x
Safe == TRUE
====