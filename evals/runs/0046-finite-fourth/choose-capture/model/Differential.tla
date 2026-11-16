---- MODULE Differential ----
EXTENDS Integers
VARIABLE x
Init == x = {CHOOSE m \in 0..3: m > n: n \in 0..2}
Next == UNCHANGED x
Safe == x = {1,2,3}
====