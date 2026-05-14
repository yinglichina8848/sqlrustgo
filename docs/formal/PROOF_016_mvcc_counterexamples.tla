--------------------------- MODULE PROOF_016_mvcc_counterexamples ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS
  T1, T2,  \* Two transaction IDs
  X, Y     \* Two database keys

VARIABLES
  store,
  versions,
  txnSnapshots,
  txnReadSet,
  aborted,
  step

vars == <<store, versions, txnSnapshots, txnReadSet, aborted, step>>

Init ==
  /\ store = [X |-> 0, Y |-> 0]
  /\ versions = [X |-> <<>>, Y |-> <<>>]
  /\ txnSnapshots = [T1 |-> 0, T2 |-> 0]
  /\ txnReadSet = [T1 |-> {}, T2 |-> {}]
  /\ aborted = {}
  /\ step = 0

\* === COUNTEREXAMPLE 1: Dirty Read ===
\* T1 writes X=1, doesn't commit
\* T2 reads X and sees uncommitted value (VIOLATION)

DirtyRead ==
  /\ step = 0
  /\ versions' = [versions EXCEPT ![X] = Append(versions[X], [value|->1, begin_ts|->T1, commit_ts|->0, tx_id|->T1])]
  /\ store' = store
  /\ step' = 1
  /\ UNCHANGED <<txnSnapshots, txnReadSet, aborted>>

T2ReadsX_Uncommitted ==
  /\ step = 1
  /\ \E i \in 1..Len(versions[X]) :
    LET v == versions[X][i] IN
    /\ v.begin_ts = T1
    /\ v.commit_ts = 0  \* Uncommitted!
    /\ txnReadSet' = [txnReadSet EXCEPT ![T2] = txnReadSet[T2] \cup {{X, v.value, v.commit_ts}}]
  /\ step' = 2
  /\ UNCHANGED <<store, versions, txnSnapshots, aborted>>

\* Expected: This state should be REJECTED (T2 cannot see uncommitted data)
\* Violation: T2 reads X=1 from uncommitted T1

\* === COUNTEREXAMPLE 2: Read Skew (Non-Repeatable Read) ===
\* T1 reads X=1 at snapshot 10
\* T2 updates X to 2 and commits at timestamp 11
\* T1 reads X again at same snapshot - should see same value

ReadSkew ==
  /\ step = 0
  /\ versions' = [versions EXCEPT
    ![X] = Append(versions[X], [value|->1, begin_ts|->10, commit_ts|->10, tx_id|->T1]),
    ![Y] = Append(versions[Y], [value|->1, begin_ts|->10, commit_ts|->10, tx_id|->T1])]
  /\ store' = [X |-> 1, Y |-> 1]
  /\ txnSnapshots' = [T1 |-> 10, T2 |-> 0]
  /\ step' = 1
  /\ UNCHANGED <<txnReadSet, aborted>>

T1ReadsFirstTime ==
  /\ step = 1
  /\ txnSnapshots[T1] = 10
  /\ \E i \in 1..Len(versions[X]) :
    LET v == versions[X][i] IN
    /\ v.commit_ts <= txnSnapshots[T1]  \* Visible at snapshot 10
    /\ txnReadSet' = [txnReadSet EXCEPT ![T1] = txnReadSet[T1] \cup {{X, v.value, v.commit_ts}}]
  /\ step' = 2
  /\ UNCHANGED <<store, versions, txnSnapshots, aborted>>

T2UpdatesX ==
  /\ step = 2
  /\ versions' = [versions EXCEPT
    ![X] = Append(versions[X], [value|->2, begin_ts|->11, commit_ts|->11, tx_id|->T2])]
  /\ store' = [X |-> 2, Y |-> 1]
  /\ step' = 3
  /\ UNCHANGED <<txnSnapshots, txnReadSet, aborted>>

T1ReadsSecondTime ==
  /\ step = 3
  /\ \E i \in 1..Len(versions[X]) :
    LET v == versions[X][i] IN
    /\ v.commit_ts <= txnSnapshots[T1]  \* Same snapshot 10 - should see same value!
    /\ txnReadSet' = [txnReadSet EXCEPT ![T1] = txnReadSet[T1] \cup {{X, v.value, v.commit_ts}}]
  /\ step' = 4
  /\ UNCHANGED <<store, versions, txnSnapshots, aborted>>

\* Expected: T1's second read should see same value (1) at snapshot 10
\* Violation: If T1 sees value 2, snapshot isolation is broken

\* === COUNTEREXAMPLE 3: Write Skew ===
\* T1 reads X=1, Y=1
\* T2 reads X=1, Y=1
\* T1 writes Y=0, T2 writes X=0
\* Both commit - but invariant X+Y >= 2 is violated

WriteSkewSetup ==
  /\ step = 0
  /\ versions' = [versions EXCEPT
    ![X] = Append(versions[X], [value|->1, begin_ts|->1, commit_ts|->1, tx_id|->T1]),
    ![Y] = Append(versions[Y], [value|->1, begin_ts|->1, commit_ts|->1, tx_id|->T1])]
  /\ store' = [X |-> 1, Y |-> 1]
  /\ txnSnapshots' = [T1 |-> 1, T2 |-> 1]
  /\ step' = 1
  /\ UNCHANGED <<txnReadSet, aborted>>

T1ReadsBoth ==
  /\ step = 1
  /\ txnReadSet' = [txnReadSet EXCEPT
    ![T1] = txnReadSet[T1] \cup {{X, 1, 1}, {Y, 1, 1}}]
  /\ step' = 2
  /\ UNCHANGED <<store, versions, txnSnapshots, aborted>>

T2ReadsBoth ==
  /\ step = 2
  /\ txnReadSet' = [txnReadSet EXCEPT
    ![T2] = txnReadSet[T2] \cup {{X, 1, 1}, {Y, 1, 1}}]
  /\ step' = 3
  /\ UNCHANGED <<store, versions, txnSnapshots, aborted>>

T1WritesY ==
  /\ step = 3
  /\ versions' = [versions EXCEPT
    ![Y] = Append(versions[Y], [value|->0, begin_ts|->3, commit_ts|->0, tx_id|->T1])]
  /\ step' = 4
  /\ UNCHANGED <<store, txnSnapshots, txnReadSet, aborted>>

T2WritesX ==
  /\ step = 4
  /\ versions' = [versions EXCEPT
    ![X] = Append(versions[X], [value|->0, begin_ts|->4, commit_ts|->0, tx_id|->T2])]
  /\ step' = 5
  /\ UNCHANGED <<store, txnSnapshots, txnReadSet, aborted>>

T1Commits ==
  /\ step = 5
  /\ aborted' = aborted
  /\ step' = 6
  /\ UNCHANGED <<store, versions, txnSnapshots, txnReadSet>>

T2Commits ==
  /\ step = 6
  /\ aborted' = aborted
  /\ step' = 7
  /\ UNCHANGED <<store, versions, txnSnapshots, txnReadSet>>

\* Business Invariant: X + Y >= 2 (at least one of X, Y is 1)
BusinessInvariant == store[X] + store[Y] >= 2

\* Expected: With SSI, write skew should be prevented
\* Violation: Final state has X=0, Y=0, invariant VIOLATED

Next ==
  \/ DirtyRead
  \/ T2ReadsX_Uncommitted
  \/ ReadSkew
  \/ T1ReadsFirstTime
  \/ T2UpdatesX
  \/ T1ReadsSecondTime
  \/ WriteSkewSetup
  \/ T1ReadsBoth
  \/ T2ReadsBoth
  \/ T1WritesY
  \/ T2WritesX
  \/ T1Commits
  \/ T2Commits

Spec == Init /\ [][Next]_vars

\* Model checking options:
\* - Check BusinessInvariant in all reachable states
\* - Check if DirtyRead state is reachable (it should NOT be in correct MVCC)
\* - Check if WriteSkew final state is reachable (it SHOULD be blocked by SSI)

============================================================================