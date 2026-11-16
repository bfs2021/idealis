---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = -8
Init == x = (-2)^3
Next == UNCHANGED x
====