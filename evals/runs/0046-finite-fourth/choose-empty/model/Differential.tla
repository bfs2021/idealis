---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = CHOOSE n \in {}: TRUE
Next == UNCHANGED x
====