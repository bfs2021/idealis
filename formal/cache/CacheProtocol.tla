---- MODULE CacheProtocol ----
EXTENDS Naturals, FiniteSets
CONSTANT Broken
VARIABLES cell, tokens, clock, bad, event
vars == <<cell, tokens, clock, bad, event>>
Workers == {"w1", "w2"}
Action(op, owner, fence, receipt, complete) ==
  [op |-> op, identity |-> "a", owner |-> owner, fence |-> fence,
   now |-> clock, duration |-> 2, receiptIdentity |-> receipt,
   complete |-> complete]
Cleared == [identity |-> "a", fence |-> cell.fence, phase |-> "idle",
            owner |-> "", expiresAt |-> 0, storedIdentity |-> "", complete |-> FALSE]
Init == /\ cell = [identity |-> "a", fence |-> 0, phase |-> "idle",
                   owner |-> "", expiresAt |-> 0, storedIdentity |-> "", complete |-> FALSE]
        /\ tokens = {}
        /\ clock = 0
        /\ bad = FALSE
        /\ event = Action("init", "", 0, "", FALSE)
Claim(worker) ==
  /\ cell.phase # "ready"
  /\ (cell.phase = "idle" \/ clock >= cell.expiresAt)
  /\ cell.fence < 2
  /\ cell' = [Cleared EXCEPT !.fence = @ + 1, !.phase = "leased",
                             !.owner = worker, !.expiresAt = clock + 2]
  /\ tokens' = tokens \cup {[owner |-> worker, fence |-> cell.fence + 1]}
  /\ event' = Action("acquire", worker, 0, "", FALSE)
  /\ UNCHANGED <<clock, bad>>
Live(token) == cell.phase = "leased" /\ cell.owner = token.owner /\ clock < cell.expiresAt
Renew(token) ==
  /\ Live(token) /\ token.fence = cell.fence
  /\ cell' = [cell EXCEPT !.expiresAt = IF @ > clock + 2 THEN @ ELSE clock + 2]
  /\ event' = Action("renew", token.owner, token.fence, "", FALSE)
  /\ UNCHANGED <<tokens, clock, bad>>
Finish(token, receipt, complete) ==
  /\ Live(token)
  /\ (Broken = 1 \/ token.fence = cell.fence)
  /\ (Broken = 2 \/ receipt = "a")
  /\ LET accepted == complete \/ Broken = 3 IN
       /\ cell' = IF accepted
            THEN [cell EXCEPT !.phase = "ready", !.owner = "", !.expiresAt = 0,
                              !.storedIdentity = receipt, !.complete = complete]
            ELSE Cleared
       /\ bad' = (bad \/ (accepted /\ (token.fence # cell.fence \/ receipt # "a" \/ ~complete)))
  /\ event' = Action("publish", token.owner, token.fence, receipt, complete)
  /\ UNCHANGED <<tokens, clock>>
Release(token) ==
  /\ cell.phase = "leased" /\ cell.owner = token.owner /\ token.fence = cell.fence
  /\ cell' = Cleared
  /\ event' = Action("release", token.owner, token.fence, "", FALSE)
  /\ UNCHANGED <<tokens, clock, bad>>
Evict == /\ cell.phase = "ready"
         /\ cell' = Cleared
         /\ event' = Action("evict", "", 0, "", FALSE)
         /\ UNCHANGED <<tokens, clock, bad>>
Tick == /\ clock < 3
        /\ clock' = clock + 1
        /\ event' = Action("tick", "", 0, "", FALSE)
        /\ UNCHANGED <<cell, tokens, bad>>
Next == (\E worker \in Workers: Claim(worker))
        \/ (\E token \in tokens: Renew(token) \/ Release(token)
             \/ (\E receipt \in {"a", "b"}, complete \in BOOLEAN: Finish(token, receipt, complete)))
        \/ Evict \/ Tick
Safe == ~bad /\ (cell.phase = "ready" => cell.storedIdentity = "a" /\ cell.complete)
NoReady == cell.phase # "ready"
====
