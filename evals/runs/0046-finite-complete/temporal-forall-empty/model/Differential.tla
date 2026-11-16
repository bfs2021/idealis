---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Live == \A n \in {}: <> FALSE
Init == x = 0
Next == UNCHANGED x
====