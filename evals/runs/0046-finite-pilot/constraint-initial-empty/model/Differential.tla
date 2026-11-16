---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Limit == x > 1
Init == x = 0
Next == x' = x + 1
====