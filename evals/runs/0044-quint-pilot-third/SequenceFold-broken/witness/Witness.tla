---- MODULE Witness ----
EXTENDS SequenceFold
Init == q_init
Next == q_step

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (total = 312)
__TraceNext == ((__position = 0 /\ __position' = 0 /\ (total' = 312))) /\ (Next \/ UNCHANGED <<total>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<total,__position>> /\ WF_<<total,__position>>(__TraceNext)
__Expected == <>(~(broken))
====