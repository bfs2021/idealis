---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x \notin SUBSET {1,2}
Init == x \in Nodes
Next == UNCHANGED x
====