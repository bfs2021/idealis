---- MODULE Differential ----
EXTENDS Integers
VARIABLE x
Init == x = CHOOSE a \in {1}: a
Next == UNCHANGED x
Safe == TRUE
====