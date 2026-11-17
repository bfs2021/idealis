---- MODULE Differential ----
EXTENDS Naturals
VARIABLE x
A == INSTANCE Actions WITH value <- x
Init == A!Init
Next == A!Advance
Safe == x < 2

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (x = 0)
__TraceNext == ((__position = 0 /\ __position' = 1 /\ (x' = 1)) \/ (__position = 1 /\ __position' = 2 /\ (x' = 2)) \/ (__position = 2 /\ __position' = 2 /\ (x' = 2))) /\ (Next \/ UNCHANGED <<x>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<x,__position>> /\ WF_<<x,__position>>(__TraceNext)
__Expected == <>(~(Safe))
====