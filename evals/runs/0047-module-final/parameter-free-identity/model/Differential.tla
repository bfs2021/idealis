---- MODULE Differential ----
EXTENDS Helper
VARIABLE x
INSTANCE Helper
Init == x = Value
Next == UNCHANGED x
Safe == x = 1
====