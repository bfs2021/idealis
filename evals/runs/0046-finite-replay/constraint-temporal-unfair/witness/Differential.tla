---- MODULE Differential ----
EXTENDS Integers
VARIABLE x
Init == x = 0
Next == x' = x + 1
Limit == x < 2
Live == <> (x = 2)
Spec == Init /\ [][Next]_x

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (x = 0)
__TraceNext == ((__position = 0 /\ __position' = 0 /\ (x' = 0))) /\ (Next \/ UNCHANGED <<x>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<x,__position>> /\ WF_<<x,__position>>(__TraceNext)
__Expected == ~(Live)
====