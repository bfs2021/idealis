---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x # "alice"
Init == x \in Nodes
Next == UNCHANGED x
====