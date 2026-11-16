---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Limit == x < 3
Safe == x < 3
Init == x = 0
Next == x' = x + 1
====