---- MODULE Differential ----
VARIABLE x
LOCAL INSTANCE Helper
Init == x = Value
Next == UNCHANGED x
Safe == x = 1
====