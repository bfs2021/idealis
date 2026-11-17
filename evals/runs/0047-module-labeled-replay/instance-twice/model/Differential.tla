---- MODULE Differential ----
EXTENDS Naturals
VARIABLES x, y
A == INSTANCE Counter WITH value <- x, Limit <- 1
B == INSTANCE Counter WITH value <- y, Limit <- 2
Init == A!Init /\ B!Init
Next == (A!Next /\ UNCHANGED y) \/ (B!Next /\ UNCHANGED x)
Safe == x <= 1 /\ y <= 2
====