---- MODULE Differential ----
EXTENDS Integers
VARIABLE x
Init == x = CHOOSE n \in {"z", "a", "b"}: n # "a"
Next == UNCHANGED x
Safe == x = "b"
====