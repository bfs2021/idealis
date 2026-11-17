---- MODULE Differential ----
EXTENDS Helper
VARIABLE x
Init == x = 0
Next == LET Advance == x' = x + 1 IN Advance
Safe == x < 2
====