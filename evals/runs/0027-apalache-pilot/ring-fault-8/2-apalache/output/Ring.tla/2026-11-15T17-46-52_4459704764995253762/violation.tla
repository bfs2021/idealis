---------------------------- MODULE counterexample ----------------------------

EXTENDS Ring

(* Constant initialization state *)
ConstInit == TRUE

(* Initial state [_transition(0)] *)
State0 == x = 0

(* State1 [_transition(0)] *)
State1 == x = 1

(* State2 [_transition(0)] *)
State2 == x = 2

(* State3 [_transition(0)] *)
State3 == x = 3

(* State4 [_transition(0)] *)
State4 == x = 4

(* State5 [_transition(0)] *)
State5 == x = 5

(* State6 [_transition(0)] *)
State6 == x = 6

(* State7 [_transition(0)] *)
State7 == x = 7

(* State8 [_transition(0)] *)
State8 == x = 8

(* The following formula holds true in the last state and violates the invariant *)
InvariantViolation == x >= 8

================================================================================
(* Created by Apalache on Sun Nov 15 17:46:53 JST 2026 *)
(* https://github.com/apalache-mc/apalache *)
