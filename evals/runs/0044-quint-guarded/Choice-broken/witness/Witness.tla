---- MODULE Witness ----
EXTENDS Choice
Init == q_init
Next == q_step

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (value = 0)
__TraceNext == ((__position = 0 /\ __position' = 1 /\ (value' = 2)) \/ (__position = 1 /\ __position' = 1 /\ (value' = 2))) /\ (Next \/ UNCHANGED <<value>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<value,__position>> /\ WF_<<value,__position>>(__TraceNext)
__Expected == <>(~(broken))
====