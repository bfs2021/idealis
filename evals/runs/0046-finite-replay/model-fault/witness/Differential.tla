---- MODULE Differential ----
EXTENDS Integers, FiniteSets, Sequences
VARIABLE x
CONSTANT Nodes
Safe == x # (CHOOSE n \in Nodes: TRUE)
Init == x \in Nodes
Next == UNCHANGED x

CONSTANT __IdealisModelValue0
VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (x = __IdealisModelValue0)
__TraceNext == ((__position = 0 /\ __position' = 0 /\ (x' = __IdealisModelValue0))) /\ (Next \/ UNCHANGED <<x>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<x,__position>> /\ WF_<<x,__position>>(__TraceNext)
__Expected == <>(~(Safe))
====