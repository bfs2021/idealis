---- MODULE Differential ----
EXTENDS Facade
VARIABLE x
Init == x = I!Value
Next == UNCHANGED x
Safe == TRUE
====