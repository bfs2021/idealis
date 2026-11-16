---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x \notin [key: BOOLEAN]
Init == x \in Nodes
Next == UNCHANGED x
====