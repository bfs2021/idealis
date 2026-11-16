---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x \notin [1..2 -> BOOLEAN]
Init == x \in Nodes
Next == UNCHANGED x
====