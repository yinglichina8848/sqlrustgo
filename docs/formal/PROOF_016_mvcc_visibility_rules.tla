--------------------------- MODULE PROOF_016_mvcc_visibility ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC, BallotCounter

CONSTANTS
  NULL,
  Tbl,
  MaxVersions  \* Maximum number of versions per key

VARIABLES
  store,          \* Current store state: [key -> value]
  versions,       \* Version chains: [key -> Seq({value, begin_ts, commit_ts, tx_id})]
  activeTxns,     \* Set of active transaction IDs
  committedTxns,  \* Set of committed transaction IDs
  txnSnapshots,   \* [txn_id -> snapshot_timestamp]
  txnReadSet,     \* [txn_id -> {(key, value, commit_ts)}]
  aborted,
  step

TypeInvariant ==
  /\ store \in [Tbl -> Nat]
  /\ versions \in [Tbl -> Seq(Nat)]
  /\ activeTxns \subseteq Nat
  /\ committedTxns \subseteq Nat
  /\ txnSnapshots \in [Nat -> Nat]
  /\ txnReadSet \in [Nat -> SUBSET (Tbl \times Nat \times Nat)]
  /\ aborted \subseteq Nat
  /\ step \in Nat

Init ==
  /\ store = [k \in Tbl |-> 0]
  /\ versions = [k \in Tbl |-> << >>]
  /\ activeTxns = {}
  /\ committedTxns = {}
  /\ txnSnapshots = [t \in Nat |-> 0]
  /\ txnReadSet = [t \in Nat |-> {}]
  /\ aborted = {}
  /\ step = 0

\* T1: Begin Transaction - assign begin_ts
BeginTxn(t) ==
  /\ t \notin activeTxns
  /\ t \notin committedTxns
  /\ activeTxns' = activeTxns \cup {t}
  /\ txnSnapshots' = [txnSnapshots EXCEPT ![t] = t]
  /\ step' = step + 1
  /\ UNCHANGED <<store, versions, committedTxns, txnReadSet, aborted>>

\* T2: Read - visibility check based on snapshot timestamp
\* Rule: A version (value, commit_ts) is visible to txn t if:
\*   1. commit_ts < txnSnapshots[t]  (committed before snapshot)
\*   2. commit_ts < t  (committed before txn began) - same as above
Read(t, k) ==
  /\ t \in activeTxns
  /\ k \in Tbl
  /\ \E i \in 1..Len(versions[k]) :
    LET v == versions[k][i] IN
    /\ v.commit_ts < t  \* Rule: committed before txn began
    /\ txnReadSet' = [txnReadSet EXCEPT ![t] = txnReadSet[t] \cup {{k, v.value, v.commit_ts}}]
  /\ step' = step + 1
  /\ UNCHANGED <<store, versions, activeTxns, committedTxns, txnSnapshots, aborted>>

\* T3: Write - create new version (not yet committed)
Write(t, k, val) ==
  /\ t \in activeTxns
  /\ k \in Tbl
  /\ versions' = [versions EXCEPT ![k] = Append(versions[k], [value |-> val, begin_ts |-> t, commit_ts |-> 0, tx_id |-> t])]
  /\ step' = step + 1
  /\ UNCHANGED <<store, activeTxns, committedTxns, txnSnapshots, txnReadSet, aborted>>

\* T4: TryCommit - validation phase
\* Rule: begin_ts < commit_ts < snapshot_ts for valid commit
TryCommit(t) ==
  /\ t \in activeTxns
  /\ \/ /\ t > txnSnapshots[t]  \* Rule: begin_ts < commit_ts
        /\ \A other \in activeTxns \setminus {t} :
          txnSnapshots[t] <= txnSnapshots[other]  \* snapshot_ts ordering
        /\ committedTxns' = committedTxns \cup {t}
        /\ activeTxns' = activeTxns \setminus {t}
        /\ aborted' = aborted
     \/ /\ \E other \in activeTxns \setminus {t} :
             \/ t < txnSnapshots[other]  \* concurrent txn started after
             \/ \E r \in txnReadSet[t] :
                \E w \in versions[r.1] :
                  w.commit_ts > txnSnapshots[t]  \* read uncommitted
        /\ aborted' = aborted \cup {t}
        /\ activeTxns' = activeTxns \setminus {t}
  /\ step' = step + 1
  /\ UNCHANGED <<store, versions, txnSnapshots, txnReadSet, committedTxns>>

\* T5: Update commit timestamps after successful commit
FinalizeCommit(t) ==
  /\ t \in committedTxns
  /\ \A k \in Tbl :
    \A i \in 1..Len(versions[k]) :
      LET v == versions[k][i] IN
        IF v.tx_id = t /\ v.commit_ts = 0 THEN
          versions' = [versions EXCEPT ![k][i].commit_ts = t]
        ELSE UNCHANGED versions
  /\ UNCHANGED <<store, activeTxns, committedTxns, txnSnapshots, txnReadSet, aborted, step>>

\* Rollback
Rollback(t) ==
  /\ t \in activeTxns
  /\ aborted' = aborted \cup {t}
  /\ activeTxns' = activeTxns \setminus {t}
  /\ step' = step + 1
  /\ UNCHANGED <<store, versions, committedTxns, txnSnapshots, txnReadSet>>

Next ==
  \/ \E t \in Nat : BeginTxn(t)
  \/ \E t \in Nat : \E k \in Tbl : Read(t, k)
  \/ \E t \in Nat : \E k \in Tbl : \E v \in Nat : Write(t, k, v)
  \/ \E t \in Nat : TryCommit(t)
  \/ \E t \in Nat : FinalizeCommit(t)
  \/ \E t \in Nat : Rollback(t)

Spec == Init /\ [][Next]_<<store, versions, activeTxns, committedTxns, txnSnapshots, txnReadSet, aborted, step>>

StepBound == step <= 200

\* Core Visibility Invariants
VisibilityRule1 ==
  \A t \in activeTxns :
    \A (k, v, cts) \in txnReadSet[t] :
      cts < t  \* Rule: commit_ts < begin_ts

VisibilityRule2 ==
  \A t \in committedTxns :
    txnSnapshots[t] <= t  \* Rule: snapshot_ts <= commit_ts

VisibilityRule3 ==
  \A t1, t2 \in committedTxns :
    t1 < t2 => txnSnapshots[t1] < txnSnapshots[t2]  \* Rule: later commits have later snapshots

\* Version Chain Invariants
VersionChainRule ==
  \A k \in Tbl :
    \A i \in 1..Len(versions[k]) :
      LET v == versions[k][i] IN
        v.commit_ts = 0 \/ v.commit_ts >= v.begin_ts  \* Rule: commit_ts >= begin_ts

NoPhantomReads ==
  \A t \in Nat :
    \A k \in Tbl :
      Cardinality({v \in txnReadSet[t] : v.1 = k}) <= 1  \* Each key read at most once

================================================================================