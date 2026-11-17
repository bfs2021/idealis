---- MODULE Differential ----
EXTENDS Left, Right
VARIABLE x
Init == x = Answer
Next == UNCHANGED x
Safe == x = 42
====