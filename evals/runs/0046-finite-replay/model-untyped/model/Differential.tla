---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x \in Nodes
Init == x \in Nodes
Next == UNCHANGED x
====