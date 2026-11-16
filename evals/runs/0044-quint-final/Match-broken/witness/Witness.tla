---- MODULE Witness ----
EXTENDS Match
Init == q_init
Next == q_step

VARIABLE __position
__TraceInit == Init /\ __position = 0 /\ (phase = [__key0 \in {"Idle"} |-> IF __key0 = "Idle" THEN [__key1 \in {"tag"} |-> IF __key1 = "tag" THEN "UNIT" ELSE TRUE] ELSE TRUE])
__TraceNext == ((__position = 0 /\ __position' = 1 /\ (phase' = [__key0 \in {"Busy"} |-> IF __key0 = "Busy" THEN [__key1 \in {"tag"} |-> IF __key1 = "tag" THEN "UNIT" ELSE TRUE] ELSE TRUE])) \/ (__position = 1 /\ __position' = 2 /\ (phase' = [__key0 \in {"Done"} |-> IF __key0 = "Done" THEN [__key1 \in {"tag"} |-> IF __key1 = "tag" THEN "UNIT" ELSE TRUE] ELSE TRUE])) \/ (__position = 2 /\ __position' = 2 /\ (phase' = [__key0 \in {"Done"} |-> IF __key0 = "Done" THEN [__key1 \in {"tag"} |-> IF __key1 = "tag" THEN "UNIT" ELSE TRUE] ELSE TRUE]))) /\ (Next \/ UNCHANGED <<phase>>)
__TraceSpec == __TraceInit /\ [][__TraceNext]_<<phase,__position>> /\ WF_<<phase,__position>>(__TraceNext)
__Expected == <>(~(broken))
====