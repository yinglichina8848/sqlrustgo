---- MODULE PROOF_023_deadlock_atomic ----
\* PROOF-023 Atomic Fix: Atomic Pre-Check + Add-Edge
\* =================================================
\*
\* PURPOSE:
\*   Prove that when "check would_create_cycle" and "add_edge" ARE atomic,
\*   NoCycle invariant is preserved regardless of thread interleaving.
\*
\* MODEL:
\*   Single atomic operation: AtomicAdd(t, targets)
\*   = Check + Commit combined (no interleaving possible)
\*
\* KEY:
\*   waitFor[t] is updated atomically — no TOCTOU window
\*
\* EXPECTED RESULT: NoCycle INVARIANT HOLDS
\* =========================================

EXTENDS Naturals, FiniteSets, TLC

CONSTANTS
  Txns

VARIABLES
  waitFor   \* [t -> SET t]: transactions t is waiting for

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
  waitFor = [t \in Txns |-> {}]

(***************************************************************************)
(* ATOMIC OPERATION: Check + Commit as single step                         *)
(*                                                                         *)
(* PRECONDITIONS:                                                          *)
(*   - targets ⊆ Txns                                                       *)
(*   - t ∉ targets                                                         *)
(*   - ∀h ∈ targets: ~Reachable(h, t)   ← pre-check                        *)
(*                                                                         *)
(* EFFECT (atomic, no TOCTOU window):                                      *)
(*   waitFor[t] := waitFor[t] ∪ targets                                    *)
(*                                                                         *)
(* Because there is no interleaving between check and commit,              *)
\* a cycle can NEVER be formed.                                            *)
(***************************************************************************)

AtomicAdd(t, targets) ==
  /\ targets \subseteq Txns
  /\ t \notin targets
  /\ \A h \in targets : ~Reachable(h, t)   \* pre-check (atomically verified)
  /\ waitFor' = [waitFor EXCEPT ![t] = @ \cup targets]

(***************************************************************************)
(* Next                                                                    *)
(***************************************************************************)

Next ==
  \E t \in Txns :
    \E targets \in SUBSET (Txns \ {t}) :
      AtomicAdd(t, targets)

Spec == Init /\ [][Next]_waitFor

(***************************************************************************)
(* SAFETY INVARIANT: No cycle in wait-for graph                           *)
(*                                                                         *)
\* With atomic Check+Commit, cycle formation is impossible:
\* - Each AtomicAdd checks before writing
\* - No interleaving can occur between check and write
\* - Therefore ~Reachable(h, t) guarantees no cycle will ever be created
(***************************************************************************)

NoCycle ==
  \A t \in Txns : ~Reachable(t, t)

=============================================================================
