---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = -1
Init == x = (-1)^2147483647
Next == UNCHANGED x
====