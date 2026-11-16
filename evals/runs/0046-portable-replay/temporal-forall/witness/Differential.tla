---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
Live == \A n \in 0..1: <> (x = n)
Init == x = 0
Next == x' = 1 - x

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (x = 0)
__TraceNext == ((__position = 0 /\ __position' = 1 /\ (x' = 0)) \/ (__position = 1 /\ __position' = 1 /\ (x' = 0))) /\ (Next \/ UNCHANGED <<x>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<x,__position>> /\ WF_<<x,__position>>(__TraceNext)
__Expected == ~(Live)
====