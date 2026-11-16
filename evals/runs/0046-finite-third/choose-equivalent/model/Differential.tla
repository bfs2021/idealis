---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = (CHOOSE n \in 1..3: n > 0)
Init == x = (CHOOSE n \in {3, 1, 2}: n > 0)
Next == UNCHANGED x
====