---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = 1 % (-2)
Next == UNCHANGED x
====