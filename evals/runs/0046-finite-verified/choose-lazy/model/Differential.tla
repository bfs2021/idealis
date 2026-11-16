---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x

Init == x = IF TRUE THEN 1 ELSE (CHOOSE n \in {}: TRUE)
Next == UNCHANGED x
====