---- MODULE Differential ----
EXTENDS Integers
VARIABLE x
Init == x = CHOOSE <<a,b>> \in {<<1>>}: TRUE
Next == UNCHANGED x
Safe == TRUE
====