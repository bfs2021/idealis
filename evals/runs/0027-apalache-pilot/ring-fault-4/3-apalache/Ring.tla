---- MODULE Ring ----
EXTENDS Naturals
VARIABLE
  \* @type: Int;
  x
Init == x = 0
Next == x' = IF x < 15 THEN x + 1 ELSE 0
Safe == x < 8
====