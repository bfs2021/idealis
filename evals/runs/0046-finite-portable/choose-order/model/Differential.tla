---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = 2
Init == x = CHOOSE n \in {3, 1, 2}: n > 1
Next == UNCHANGED x
====