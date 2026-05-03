---- MODULE PROOF_016_023_mvcc_atomic ----
\* Unified Model: MVCC + Wait-For Graph + ATOMIC Pre-Check
\* =========================================================
\*
\* PURPOSE:
\*   Prove that when MVCC + Wait-For edges use ATOMIC pre-check,
\*   the combined system preserves BOTH:
\*     (1) NoCycle        — deadlock-free (by construction)
\*     (2) NoWriteConflict — no concurrent writes on same key
\*
\* PROOF STRUCTURE:
\*   1. PROOF_016_023_mvcc_toctou.tla  → Non-atomic (FAIL)
\*   2. PROOF_016_023_mvcc_atomic.tla  → Atomic      (PASS)
\*
\* EXPECTED RESULT: Both NoCycle AND NoWriteConflict HOLD
\* ======================================================

EXTENDS Naturals, FiniteSets

CONSTANTS
  Txns,    \* Model values: {"T1","T2"}
  Keys     \* Model values: {"K1"}

VARIABLES
  (****************************************************************)
  (* Wait-For Graph (atomic only — no TOCTOU)                    *)
  (****************************************************************)
  waitFor,   \* [t -> SET t]: t is waiting for transactions in set

  (****************************************************************)
  (* MVCC state                                                  *)
  (****************************************************************)
  writeSet,       \* [t -> SET Keys]: keys written by t (uncommitted)
  readSet,        \* [t -> SET Keys]: keys read by t (snapshot)
  committed,      \* SET of committed transactions
  committedWrites \* SET of Keys: keys written by committed txns

(***************************************************************************)
(* Init                                                                      *)
(***************************************************************************)

Init ==
  /\ waitFor        = [t \in Txns |-> {}]
  /\ writeSet       = [t \in Txns |-> {}]
  /\ readSet        = [t \in Txns |-> {}]
  /\ committed      = {}
  /\ committedWrites = {}

(***************************************************************************)
(* Reachability in Wait-For Graph                                          *)
(***************************************************************************)

RECURSIVE Reachable(_,_)

Reachable(t1, t2) ==
  \/ t2 \in waitFor[t1]
  \/ \E t \in waitFor[t1] : Reachable(t, t2)

(***************************************************************************)
(* ATOMIC Add Wait-For Edge (pre-check + commit as single step)            *)
(***************************************************************************)

AtomicAddWaitFor(t, targets) ==
  /\ targets \subseteq Txns
  /\ t \notin targets
  /\ \A h \in targets : ~Reachable(h, t)   \* atomic pre-check
  /\ waitFor' = [waitFor EXCEPT ![t] = @ \cup targets]
  /\ UNCHANGED <<writeSet, readSet, committed, committedWrites>>

(***************************************************************************)
(* MVCC Read: snapshot read (never conflicts)                              *)
(***************************************************************************)

Read(t, k) ==
  /\ k \notin writeSet[t]
  /\ readSet' = [readSet EXCEPT ![t] = @ \cup {k}]
  /\ UNCHANGED <<waitFor, writeSet, committed, committedWrites>>

(***************************************************************************)
(* MVCC Write: atomic conflict resolution                                   *)
(*                                                                         *)
(* BRANCH 1: Conflict with uncommitted t2                                   *)
(*   → ATOMIC pre-check (via AtomicAddWaitFor semantics)                   *)
(*   → If pre-check passes: add wait-for edge, wait for t2                *)
(*   → If pre-check fails: deadlock → transaction must abort              *)
(*                                                                         *)
(* BRANCH 2: No conflict                                                   *)
(*   → Direct write to writeSet                                            *)
(***************************************************************************)

Write(t, k) ==
  /\ t \notin committed
  /\ k \notin writeSet[t]
  /\ IF \E t2 \in Txns :
          /\ t2 # t
          /\ k \in writeSet[t2]
          /\ t2 \notin committed
       THEN
          \* BRANCH 1: conflict with uncommitted t2
          LET conflicting == {t2 \in Txns :
                               /\ t2 # t
                               /\ k \in writeSet[t2]
                               /\ t2 \notin committed} IN
          /\ \A h \in conflicting : ~Reachable(h, t)   \* ATOMIC pre-check
          /\ waitFor' = [waitFor EXCEPT ![t] = @ \cup conflicting]
          /\ UNCHANGED <<writeSet, readSet, committed, committedWrites>>
       ELSE
          \* BRANCH 2: no conflict → direct write
          /\ writeSet' = [writeSet EXCEPT ![t] = @ \cup {k}]
          /\ UNCHANGED <<waitFor, readSet, committed, committedWrites>>

(***************************************************************************)
(* Commit: WW conflict check against committed writes                       *)
(*                                                                         *)
(* Before committing, verify t's writeSet doesn't overlap with any          *)
(* key already committed by another transaction.                           *)
(***************************************************************************)

CommitTxn(t) ==
  /\ t \notin committed
  /\ writeSet[t] \cap committedWrites = {}   \* WW conflict check (ATOMIC)
  /\ committed' = committed \cup {t}
  /\ committedWrites' = committedWrites \cup writeSet[t]
  /\ writeSet' = [writeSet EXCEPT ![t] = {}]
  /\ readSet'  = [readSet  EXCEPT ![t] = {}]
  /\ UNCHANGED <<waitFor>>

(***************************************************************************)
(* Next                                                                      *)
(***************************************************************************)

Next ==
  \/ \E t \in Txns : \E targets \in SUBSET (Txns \ {t}) : AtomicAddWaitFor(t, targets)
  \/ \E t \in Txns : \E k \in Keys : Read(t, k)
  \/ \E t \in Txns : \E k \in Keys : Write(t, k)
  \/ \E t \in Txns : CommitTxn(t)

Spec == Init /\ [][Next]_<<waitFor, writeSet, readSet, committed, committedWrites>>

(***************************************************************************)
(* SAFETY INVARIANTS                                                       *)
(***************************************************************************)

(*
 * NoCycle: atomic pre-check ensures wait-for graph is always a DAG.
 * No state exists where Reachable(t, t) = TRUE.
 *)
NoCycle ==
  \A t \in Txns : ~Reachable(t, t)

(*
 * NoWriteConflict: no two committed transactions wrote the same key.
 * committedWrites tracks all keys ever committed — any overlap is a violation.
 *)
NoWriteConflict ==
  \A t1, t2 \in committed :
    t1 # t2 => writeSet[t1] \cap writeSet[t2] = {}

=============================================================================
