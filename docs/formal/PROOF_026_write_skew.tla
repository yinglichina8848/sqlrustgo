--------------------------- MODULE PROOF_026_write_skew ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
  NULL,
  T1, T2,    \* Two concurrent transactions
  XKey, YKey \* Two database keys

VARIABLES
  store,
  txn1Status, txn2Status,
  txn1Read, txn2Read,
  txn1Write, txn2Write

vars == <<store, txn1Status, txn2Status, txn1Read, txn2Read, txn1Write, txn2Write>>

\* Initial state: both values are 1
Init ==
  /\ store = [XKey |-> 1, YKey |-> 1]
  /\ txn1Status = "idle" /\ txn2Status = "idle"
  /\ txn1Read = {} /\ txn2Read = {}
  /\ txn1Write = {} /\ txn2Write = {}

\* Txn 1: READ X, then WRITE Y if X = 1
Txn1ReadX ==
  /\ txn1Status = "active"
  /\ store[XKey] = 1
  /\ txn1Read' = txn1Read \cup {XKey}
  /\ UNCHANGED <<store, txn2Status, txn2Read, txn2Write>>

Txn1WriteY ==
  /\ txn1Status = "active"
  /\ XKey \in txn1Read
  /\ txn1Write' = txn1Write \cup {YKey}
  /\ UNCHANGED <<store, txn2Status, txn1Read, txn2Read, txn2Write>>

\* Txn 2: READ Y, then WRITE X if Y = 1
Txn2ReadY ==
  /\ txn2Status = "active"
  /\ store[YKey] = 1
  /\ txn2Read' = txn2Read \cup {YKey}
  /\ UNCHANGED <<store, txn1Status, txn1Read, txn1Write>>

Txn2WriteX ==
  /\ txn2Status = "active"
  /\ YKey \in txn2Read
  /\ txn2Write' = txn2Write \cup {XKey}
  /\ UNCHANGED <<store, txn1Status, txn1Read, txn1Write>>

\* Commit: apply writes to store if no conflict
TxnCommit(txn, writes) ==
  /\ (txn = T1 /\ txn1Status = "active") \/ (txn = T2 /\ txn2Status = "active")
  /\ [](* Check for SI conflict *)
     store' = [store EXCEPT
       ![k] = IF k \in writes THEN
         CASE k = XKey -> 0
           [] k = YKey -> 0
         ELSE @
       ELSE @]
  /\ IF txn = T1 THEN txn1Status' = "committed" ELSE txn2Status' = "committed"

\* The Write Skew violation: two transactions read different keys
\* and write the other key, maintaining the invariant (both >= 1)
\* but neither sees the other's write before committing
\* Result: invariant (store[X] = 1 OR store[Y] = 1) is VIOLATED

Next ==
  \/ Txn1ReadX \/ Txn2ReadY
  \/ Txn1WriteY \/ Txn2WriteX
  \/ TxnCommit(T1, txn1Write)
  \/ TxnCommit(T2, txn2Write)
  \/ Txn1WriteY \* serial equivalent (no conflict)
  \* SSI would detect: txn1Read X, txn2Read Y
  \* txn1Write Y, txn2Write X
  \* Cycle: txn1 -> txn2 via X, txn2 -> txn1 via Y

\* Invariant: at least one of X or Y is still 1
\* This is the "at least one copy" business invariant
Invariant == store[XKey] >= 1 \/ store[YKey] >= 1

\* Write Skew is NOT prevented by SI
\* SSI (Serializable Snapshot Isolation) prevents it by detecting rw-rw conflicts
SSIPreventsWriteSkew ==
  [](IF (txn1Read \cap txn2Write /= {} \/ txn2Read \cap txn1Write /= {})
     THEN ~(txn1Status = "committed" /\ txn2Status = "committed")
     ELSE TRUE)

\* Model checker: check if Invariant holds in all reachable states
\* If Invariant is VIOLATED, Write Skew occurred
\* SSIPreventsWriteSkew should make Invariant hold

============================================================================
