---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = (-2147483647 - 1)
Init == x = (-2)^31
Next == UNCHANGED x
====