---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = -3
Init == x = (-7) \div 3
Next == UNCHANGED x
====