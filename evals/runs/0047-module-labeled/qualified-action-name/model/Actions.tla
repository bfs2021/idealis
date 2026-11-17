---- MODULE Actions ----
EXTENDS Naturals
VARIABLE value
Init == value = 0
Advance == value' = value + 1
====