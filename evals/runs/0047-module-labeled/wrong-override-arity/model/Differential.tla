---- MODULE Differential ----
CONSTANT F(_)
VARIABLE x
Value == 0
Init == x = F(1)
Next == UNCHANGED x
Safe == TRUE
====