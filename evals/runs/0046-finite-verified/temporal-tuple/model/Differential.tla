---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Live == \A <<a, b>> \in {<<0, 1>>}: [] (x = a) \/ <> (x = b)
Init == x = 0
Next == UNCHANGED x
====