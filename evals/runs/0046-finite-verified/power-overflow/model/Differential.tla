---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = 2^31
Next == UNCHANGED x
====