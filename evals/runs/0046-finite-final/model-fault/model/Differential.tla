---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x # (CHOOSE n \in Nodes: TRUE)
Init == x \in Nodes
Next == UNCHANGED x
====