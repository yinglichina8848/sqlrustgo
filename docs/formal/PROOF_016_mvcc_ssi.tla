--------------------------- MODULE PROOF_016_mvcc_ssi ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
  NULL,
  Tbl

VARIABLES
  store,
  txnStatus,
  txnReadSet,
  txnWriteSet,
  aborted,
  commitLog,
  step

TypeInvariant ==
  /\ store \in [Tbl -> Nat]
  /\ txnStatus \in [Tbl -> {"idle", "active", "committing", "aborting", "committed", "aborted"}]
  /\ txnReadSet \in [Tbl -> SUBSET Tbl]
  /\ txnWriteSet \in [Tbl -> SUBSET Tbl]
  /\ aborted \subseteq Tbl
  /\ commitLog \in Seq(Tbl)
  /\ step \in Nat

Init ==
  /\ store = [k \in Tbl |-> 0]
  /\ txnStatus = [t \in Tbl |-> "idle"]
  /\ txnReadSet = [t \in Tbl |-> {}]
  /\ txnWriteSet = [t \in Tbl |-> {}]
  /\ aborted = {}
  /\ commitLog = << >>
  /\ step = 0

BeginRead(t) ==
  /\ txnStatus[t] = "idle"
  /\ txnStatus' = [txnStatus EXCEPT ![t] = "active"]
  /\ step' = step + 1
  /\ UNCHANGED <<store, txnReadSet, txnWriteSet, aborted, commitLog>>

Read(t, k) ==
  /\ txnStatus[t] = "active"
  /\ k \in Tbl
  /\ txnReadSet' = [txnReadSet EXCEPT ![t] = txnReadSet[t] \cup {k}]
  /\ step' = step + 1
  /\ UNCHANGED <<store, txnStatus, txnWriteSet, aborted, commitLog>>

Write(t, k) ==
  /\ txnStatus[t] = "active"
  /\ k \in Tbl
  /\ txnWriteSet' = [txnWriteSet EXCEPT ![t] = txnWriteSet[t] \cup {k}]
  /\ store' = [store EXCEPT ![k] = store[k] + 1]
  /\ step' = step + 1
  /\ UNCHANGED <<txnStatus, txnReadSet, aborted, commitLog>>

TryCommit(t) ==
  /\ txnStatus[t] = "active"
  /\ \/ /\ \A other \in Tbl :
             /\ other /= t
             /\ txnStatus[other] \in {"active", "committing"}
             => txnReadSet[t] \cap txnWriteSet[other] = {}
       /\ txnStatus' = [txnStatus EXCEPT ![t] = "committed"]
       /\ commitLog' = Append(commitLog, t)
       /\ aborted' = aborted
    \/ /\ \E other \in Tbl :
            /\ other /= t
            /\ txnStatus[other] \in {"active", "committing"}
            /\ txnReadSet[t] \cap txnWriteSet[other] /= {}
       /\ txnStatus' = [txnStatus EXCEPT ![t] = "aborting"]
       /\ aborted' = aborted \cup {t}
  /\ step' = step + 1
  /\ UNCHANGED <<store, txnReadSet, txnWriteSet, commitLog>>

Reset(t) ==
  /\ txnStatus[t] \in {"committed", "aborted"}
  /\ txnStatus' = [txnStatus EXCEPT ![t] = "idle"]
  /\ txnReadSet' = [txnReadSet EXCEPT ![t] = {}]
  /\ txnWriteSet' = [txnWriteSet EXCEPT ![t] = {}]
  /\ step' = step + 1
  /\ UNCHANGED <<store, aborted, commitLog>>

Next ==
  \/ \E t \in Tbl : BeginRead(t)
  \/ \E t \in Tbl : \E k \in Tbl : Read(t, k)
  \/ \E t \in Tbl : \E k \in Tbl : Write(t, k)
  \/ \E t \in Tbl : TryCommit(t)
  \/ \E t \in Tbl : Reset(t)

Spec == Init /\ [][Next]_<<store, txnStatus, txnReadSet, txnWriteSet, aborted, commitLog, step>>

StepBound == step <= 100

ConflictDetection ==
  \A t \in Tbl :
    t \in aborted
      => \E other \in Tbl :
           /\ other /= t
           /\ txnReadSet[t] \cap txnWriteSet[other] /= {}

=============================================================================
