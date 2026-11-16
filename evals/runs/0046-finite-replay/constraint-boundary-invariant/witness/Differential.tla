---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Limit == x < 3
Safe == x < 3
Init == x = 0
Next == x' = x + 1

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (x = 0)
__TraceNext == ((__position = 0 /\ __position' = 1 /\ (x' = 1)) \/ (__position = 1 /\ __position' = 2 /\ (x' = 2)) \/ (__position = 2 /\ __position' = 3 /\ (x' = 3)) \/ (__position = 3 /\ __position' = 3 /\ (x' = 3))) /\ (Next \/ UNCHANGED <<x>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<x,__position>> /\ WF_<<x,__position>>(__TraceNext)
__Expected == <>(~(Safe))
====