---- MODULE PROOF_023_deadlock_toctou ----
\* PROOF-023 TOCTOU Analysis: Non-Atomic Pre-Check
\* ================================================
\*
\* PURPOSE:
\*   Prove that when "check would_create_cycle" and "add_edge" are NOT atomic,
\*   a TOCTOU race can produce a deadlock cycle even though each transaction
\*   individually passed the Reachable check.
\*
\* MODEL:
\*   Two-phase model: Check(t, targets) → Commit(t)
\*   Check: verifies ~Reachable(h, t) for all h in targets
\*   Commit: actually adds edges to waitFor graph
\*
\*   Interleavings allow:
\*     T1: Check({T2})  passes
\*     T2: Check({T1})  passes
\*     T1: Commit       adds T1→T2
\*     T2: Commit       adds T2→T1  → CYCLE
\*
\* EXPECTED RESULT: Invariant NoCycle VIOLATED
\* ============================================

EXTENDS Naturals, FiniteSets, TLC

CONSTANTS
  Txns

VARIABLES
  waitFor,   \* [t -> SET t]: transactions t is waiting for
  phase,     \* [t -> {"idle", "checked"}]: check/commit phase
  pending    \* [t -> SET t]: edges pending commit for t

(***************************************************************************)
(* Helper: Reachability in waitFor graph                                   *)
(***************************************************************************)

RECURSIVE Reachable(_,_)

Reachable(t1, t2) ==
  \/ t2 \in waitFor[t1]
  \/ \E t \in waitFor[t1] : Reachable(t, t2)

(***************************************************************************)
(* Init state                                                              *)
(***************************************************************************)

Init ==
  /\ waitFor   = [t \in Txns |-> {}]
  /\ phase     = [t \in Txns |-> "idle"]
  /\ pending   = [t \in Txns |-> {}]

(***************************************************************************)
(* Phase 1: Check — verifies no cycle would be created                   *)
(* PRECONDITIONS:                                                          *)
(*   - t is idle (hasn't checked yet)                                      *)
(*   - targets ⊆ Txns                                                       *)
(*   - t ∉ targets (no self-wait)                                          *)
(*   - ∀h ∈ targets: ~Reachable(h, t)  ← passes pre-check                  *)
(* EFFECT:                                                                 *)
(*   phase[t] := "checked"                                                 *)
(*   pending[t] := targets  (edges staged for commit)                       *)
(***************************************************************************)

Check(t, targets) ==
  /\ phase[t] = "idle"
  /\ targets \subseteq Txns
  /\ t \notin targets
  /\ \A h \in targets : ~Reachable(h, t)   \* ← pre-check passes
  /\ phase'    = [phase   EXCEPT ![t] = "checked"]
  /\ pending'  = [pending EXCEPT ![t] = targets]
  /\ UNCHANGED waitFor

(***************************************************************************)
(* Phase 2: Commit — actually adds the staged edges to waitFor             *)
(* TOCTOU WINDOW between Check and Commit allows interleaving that         *)
(* produces a cycle (see InvariantViolation trace).                       *)
(***************************************************************************)

Commit(t) ==
  /\ phase[t] = "checked"
  /\ waitFor' = [waitFor EXCEPT ![t] = @ \cup pending[t]]
  /\ phase'   = [phase   EXCEPT ![t] = "idle"]
  /\ pending' = [pending EXCEPT ![t] = {}]

(***************************************************************************)
(* Next                                                                    *)
(***************************************************************************)

Next ==
  \/ \E t \in Txns :
       \E targets \in SUBSET (Txns \ {t}) :
         Check(t, targets)
  \/ \E t \in Txns : Commit(t)

Spec == Init /\ [][Next]_<<waitFor, phase, pending>>

(***************************************************************************)
(* SAFETY INVARIANT: No cycle in wait-for graph                           *)
\* VIOLATED by TOCTOU interleaving:
\*   T1: Check({T2}) → T2: Check({T1}) → T1: Commit → T2: Commit → CYCLE
(***************************************************************************)

NoCycle ==
  \A t \in Txns : ~Reachable(t, t)

=============================================================================
