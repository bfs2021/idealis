---- MODULE Differential ----
EXTENDS Facade
VARIABLE x
Init == x = Value
Next == UNCHANGED x
Safe == x = 1
====