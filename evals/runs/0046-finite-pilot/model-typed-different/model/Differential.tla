---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == TRUE
Init == x \in Nodes
Next == UNCHANGED x
====