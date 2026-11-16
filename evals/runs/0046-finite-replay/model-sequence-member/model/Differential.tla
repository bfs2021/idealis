---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x \notin Seq(BOOLEAN)
Init == x \in Nodes
Next == UNCHANGED x
====