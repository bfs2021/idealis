---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == (CHOOSE n \in Nodes: TRUE) = (CHOOSE n \in {x} \cup Nodes: TRUE)
Init == x \in Nodes
Next == UNCHANGED x
====