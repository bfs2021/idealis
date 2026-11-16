---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x # 1
Init == x \in Nodes
Next == UNCHANGED x
====