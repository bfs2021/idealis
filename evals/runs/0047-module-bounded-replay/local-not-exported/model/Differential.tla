---- MODULE Differential ----
EXTENDS Values
VARIABLE x
Init == x = Hidden
Next == UNCHANGED x
Safe == TRUE
====