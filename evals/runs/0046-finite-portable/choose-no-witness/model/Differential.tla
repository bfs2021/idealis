---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = CHOOSE n \in 1..3: n > 9
Next == UNCHANGED x
====