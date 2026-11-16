---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = 1
Init == x = 9^0
Next == UNCHANGED x
====