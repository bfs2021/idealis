---- MODULE Functions ----
EXTENDS Naturals
VARIABLE
  \* @type: Int -> Int;
  f
Init == f \in [1..8 -> 0..1]
Next == \E i \in 1..8: f' = [f EXCEPT ![i] = 1 - @]
Safe == f[1] = 0
====