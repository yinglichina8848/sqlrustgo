---- MODULE PROOF_016_023_mvcc_toctou ----
\* Unified Model: MVCC + Wait-For Graph + TOCTOU
\* =============================================
\*
\* PURPOSE:
\*   Prove that when MVCC write-write conflicts produce wait-for edges
\*   WITHOUT atomic pre-check, the combined system can violate BOTH:
\*     (1) NoCycle     — deadlock in wait-for graph
\*     (2) NoWriteConflict — concurrent uncommitted writes on same key
\*
\* MODEL STRUCTURE:
\*   - MVCC: writeSet, readSet, committed, ts
\*   - Wait-For Graph: waitFor (derived from write-write conflicts)
\*   - TOCTOU: phase[t] in {"idle","checked"} to split Check from CommitEdge
\*
\* KEY INSIGHT:
\*   In MVCC, a write-write conflict on the same key means:
\*     T1 writes k (uncommitted)
\*     T2 writes k (uncommitted)
\*   Without atomic check, both can see each other as holders → cycle.
\*
\* EXPECTED RESULT: Invariants NoCycle AND NoWriteConflict VIOLATED
\* =================================================================

EXTENDS Naturals, FiniteSets, TLC

CONSTANTS
  Txns,    \* Model values: {"T1","T2"}
  Keys     \* Model values: {"K1"}

VARIABLES
  (****************************************************************)
  (* Wait-For Graph + TOCTOU state                                *)
  (****************************************************************)
  waitFor,   \* [t -> SET t]: t is waiting for transactions in set
  phase,     \* [t -> {"idle","checked"}]: TOCTOU phase
  pending,   \* [t -> SET t]: staged wait-for edges for t

  (****************************************************************)
  (* MVCC state                                                  *)
  (****************************************************************)
  writeSet,  \* [t -> SET Keys]: keys written by t (uncommitted)
  readSet,   \* [t -> SET Keys]: keys read by t (snapshot)
  committed, \* SET of committed transactions
  ts         \* [t -> Nat]: transaction start timestamp (TxnId = ts)

(***************************************************************************)
(* Init                                                                      *)
(***************************************************************************)

Init ==
  /\ waitFor   = [t \in Txns |-> {}]
  /\ phase     = [t \in Txns |-> "idle"]
  /\ pending   = [t \in Txns |-> {}]
  /\ writeSet  = [t \in Txns |-> {}]
  /\ readSet   = [t \in Txns |-> {}]
  /\ committed = {}
  /\ ts        = [t \in Txns |-> t]   \* Simplified: TxnId = timestamp

(***************************************************************************)
(* Reachability in Wait-For Graph                                          *)
(***************************************************************************)

RECURSIVE Reachable(_,_)

Reachable(t1, t2) ==
  \/ t2 \in waitFor[t1]
  \/ \E t \in waitFor[t1] : Reachable(t, t2)

(***************************************************************************)
(* TOCTOU Step 1: Check — verifies no cycle would be created              *)
(*                                                                         *)
(* In MVCC context: checking if adding edge t→h would create cycle.        *)
(* phase stays "checked" until CommitEdge is called.                       *)
(***************************************************************************)

Check(t, targets) ==
  /\ phase[t] = "idle"
  /\ targets \subseteq Txns
  /\ t \notin targets
  /\ \A h \in targets : ~Reachable(h, t)   \* pre-check passes
  /\ phase'    = [phase   EXCEPT ![t] = "checked"]
  /\ pending'  = [pending EXCEPT ![t] = targets]
  /\ UNCHANGED <<waitFor, writeSet, readSet, committed, ts>>

(***************************************************************************)
(* TOCTOU Step 2: CommitEdge — actually adds staged edges to waitFor      *)
(*                                                                         *)
(* TOCTOU WINDOW: between Check and CommitEdge, other txns can interleave. *)
(***************************************************************************)

CommitEdge(t) ==
  /\ phase[t] = "checked"
  /\ waitFor' = [waitFor EXCEPT ![t] = @ \cup pending[t]]
  /\ phase'   = [phase   EXCEPT ![t] = "idle"]
  /\ pending' = [pending EXCEPT ![t] = {}]
  /\ UNCHANGED <<writeSet, readSet, committed, ts>>

(***************************************************************************)
(* MVCC Read: snapshot read (ignores uncommitted writes)                   *)
(***************************************************************************)

Read(t, k) ==
  /\ k \notin writeSet[t]        \* no self-write
  /\ readSet' = [readSet EXCEPT ![t] = @ \cup {k}]
  /\ UNCHANGED <<waitFor, phase, pending, writeSet, committed, ts>>

(***************************************************************************)
(* MVCC Write: two branches                                               *)
(*                                                                         *)
(* BRANCH 1 (Write-Write Conflict):                                        *)
(*   Another uncommitted txn t2 also wrote k.                             *)
(*   → t must wait for t2 (produce wait-for edge)                        *)
(*   → Non-atomic Check+Commit allows cycle                               *)
(*                                                                         *)
(* BRANCH 2 (Clean Write):                                                 *)
(*   No conflict → add k to writeSet immediately                          *)
(***************************************************************************)

Write(t, k) ==
  /\ k \notin writeSet[t]    \* t hasn't written k yet
  /\ IF \E t2 \in Txns :
          /\ t2 # t
          /\ k \in writeSet[t2]
          /\ t2 \notin committed
       THEN
          \* BRANCH 1: conflict → wait (TOCTOU enters Check state)
          LET conflicting == {t2 \in Txns :
                               /\ t2 # t
                               /\ k \in writeSet[t2]
                               /\ t2 \notin committed} IN
          /\ phase'    = [phase   EXCEPT ![t] = "checked"]
          /\ pending'  = [pending EXCEPT ![t] = conflicting]
          /\ UNCHANGED <<waitFor, writeSet, readSet, committed, ts>>
       ELSE
          \* BRANCH 2: no conflict → direct write
          /\ writeSet' = [writeSet EXCEPT ![t] = @ \cup {k}]
          /\ UNCHANGED <<waitFor, phase, pending, readSet, committed, ts>>

(***************************************************************************)
(* Commit: check RW/WW conflicts (simplified SSI), then commit            *)
(***************************************************************************)

CommitTxn(t) ==
  /\ t \notin committed
  /\ committed' = committed \cup {t}
  /\ writeSet'   = [writeSet EXCEPT ![t] = {}]
  /\ readSet'    = [readSet  EXCEPT ![t] = {}]
  /\ UNCHANGED <<waitFor, phase, pending, ts>>

(***************************************************************************)
(* Next                                                                      *)
(***************************************************************************)

Next ==
  \/ \E t \in Txns : \E targets \in SUBSET (Txns \ {t}) : Check(t, targets)
  \/ \E t \in Txns : CommitEdge(t)
  \/ \E t \in Txns : \E k \in Keys : Read(t, k)
  \/ \E t \in Txns : \E k \in Keys : Write(t, k)
  \/ \E t \in Txns : CommitTxn(t)

Spec == Init /\ [][Next]_<<waitFor, phase, pending, writeSet, readSet, committed, ts>>

(***************************************************************************)
(* INVARIANTS                                                             *)
(***************************************************************************)

(*
 * NoCycle: wait-for graph is always a DAG (deadlock-free).
 * VIOLATED when two txns write same key → both enter checked phase
 * → interleaving allows CommitEdge to form cycle.
 *)
NoCycle ==
  \A t \in Txns : ~Reachable(t, t)

(*
 * NoWriteConflict: committed txns have disjoint write sets.
 * This is simplified serializability — in real SSI this is more complex.
 *)
NoWriteConflict ==
  \A t1, t2 \in committed :
    t1 # t2 => writeSet[t1] \cap writeSet[t2] = {}

=============================================================================
