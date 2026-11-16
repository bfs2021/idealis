---- MODULE Witness ----
EXTENDS Counter
Init == q_init
Next == q_step

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (count = 0)
__TraceNext == ((__position = 0 /\ __position' = 1 /\ (count' = 1)) \/ (__position = 1 /\ __position' = 2 /\ (count' = 2)) \/ (__position = 2 /\ __position' = 2 /\ (count' = 2))) /\ (Next \/ UNCHANGED <<count>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<count,__position>> /\ WF_<<count,__position>>(__TraceNext)
__Expected == <>(~(broken))
====