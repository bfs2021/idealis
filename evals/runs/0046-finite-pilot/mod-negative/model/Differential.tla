---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = 2
Init == x = (-7) % 3
Next == UNCHANGED x
====