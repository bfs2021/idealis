---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = 2^(-1)
Next == UNCHANGED x
====