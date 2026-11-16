---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == [n \in Nodes |-> n][x] = x
Init == x \in Nodes
Next == UNCHANGED x
====