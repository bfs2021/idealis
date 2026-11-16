---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = 2
Init == x = CHOOSE n \in 1..3: (CHOOSE m \in 1..3: m > n) = 3
Next == UNCHANGED x
====