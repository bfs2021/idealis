---- MODULE Differential ----
EXTENDS Naturals
VARIABLE x
A == INSTANCE Actions WITH value <- x
Init == A!Init
Next == A!Advance
Safe == x < 2
====