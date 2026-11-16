---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = -2147483648
Init == x = (-2)^31
Next == UNCHANGED x
====