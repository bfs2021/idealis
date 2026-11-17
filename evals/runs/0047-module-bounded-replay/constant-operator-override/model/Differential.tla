---- MODULE Differential ----
EXTENDS Naturals
CONSTANT Limit, Add(_)
VARIABLE x
Bound == 2
Inc(v) == v + 1
Init == x = 0
Next == x < Limit /\ x' = Add(x)
Safe == x <= 2
====