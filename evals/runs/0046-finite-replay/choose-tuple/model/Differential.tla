---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Safe == x = <<1, 3>>
Init == x = CHOOSE <<a, b>> \in {<<2, 0>>, <<1, 3>>}: a < b
Next == UNCHANGED x
====