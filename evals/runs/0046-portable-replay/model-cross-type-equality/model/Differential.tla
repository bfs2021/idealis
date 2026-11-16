---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x # (CHOOSE n \in Nodes: n # x)
Init == x \in Nodes
Next == UNCHANGED x
====