---------------------------- MODULE counterexample ----------------------------

EXTENDS Functions

(* Constant initialization state *)
ConstInit == TRUE

(* Initial state [_transition(0)] *)
State0 ==
  f
      = SetAsFun({ <<1, 1>>,
        <<2, 0>>,
        <<3, 0>>,
        <<4, 0>>,
        <<5, 0>>,
        <<6, 0>>,
        <<7, 0>>,
        <<8, 0>> })

(* The following formula holds true in the last state and violates the invariant *)
InvariantViolation == ~(f[1] = 0)

================================================================================
(* Created by Apalache on Sun Nov 15 17:47:05 JST 2026 *)
(* https://github.com/apalache-mc/apalache *)
