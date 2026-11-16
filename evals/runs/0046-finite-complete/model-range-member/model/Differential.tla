---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x \notin 1..3
Init == x \in Nodes
Next == UNCHANGED x
====