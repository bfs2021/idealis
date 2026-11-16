---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = IF TRUE THEN 1 ELSE 0^0
Next == UNCHANGED x
====