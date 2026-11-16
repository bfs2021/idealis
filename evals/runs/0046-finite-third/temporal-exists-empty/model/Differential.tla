---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Live == \E n \in {}: <> TRUE
Init == x = 0
Next == UNCHANGED x
====