---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = (-2147483647 - 1) \div (-1)
Next == UNCHANGED x
====