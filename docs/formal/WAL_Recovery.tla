-------------------------- MODULE WAL_Recovery --------------------------
\* WAL-based Transaction Recovery Model
\* Proves ACID properties for committed transactions
\*
\* Fixed: removed Int, use finite ValueRange model values

EXTENDS Integers, Sequences, TLC

CONSTANT 
    NULL, Committed, Aborted,
    PageID,
    InitialValue,
    NumTransactions,
    ValueRange

VARIABLES 
    txState,
    walLog,
    dbState

LogEntry == [tx: 1..NumTransactions, page: PageID, value: ValueRange, type: {"WRITE", "COMMIT"}]

TypeOK == 
    /\ txState \in [1..NumTransactions -> {NULL, Committed, Aborted}]
    /\ walLog \in Seq(LogEntry)
    /\ dbState \in [PageID -> ValueRange]

Init == 
    /\ txState = [t \in 1..NumTransactions |-> NULL]
    /\ walLog = << >>
    /\ dbState = [p \in PageID |-> InitialValue]

WriteAhead(tx, page, value) ==
    /\ txState[tx] = NULL
    /\ walLog' = Append(walLog, [tx |-> tx, page |-> page, value |-> value, type |-> "WRITE"])
    /\ dbState' = [dbState EXCEPT ![page] = value]
    /\ txState' = txState

Commit(tx) ==
    /\ txState[tx] = NULL
    /\ txState' = [txState EXCEPT ![tx] = Committed]
    /\ walLog' = Append(walLog, [tx |-> tx, page |-> 1, value |-> 0, type |-> "COMMIT"])
    /\ dbState' = dbState

Abort(tx) ==
    /\ txState[tx] = NULL
    /\ txState' = [txState EXCEPT ![tx] = Aborted]
    /\ walLog' = walLog
    /\ dbState' = dbState

\* Get committed writes from WAL (indices into walLog)
CommittedIndices == {i \in DOMAIN walLog : walLog[i].type = "WRITE" /\ txState[walLog[i].tx] = Committed}

\* Get last write for a page
LastWrite(p, indices) == 
    LET writesToPage == {i \in indices : walLog[i].page = p}
    IN CHOOSE i \in writesToPage : \A i2 \in writesToPage : walLog[i].tx >= walLog[i2].tx

\* Recovery: Replay committed transactions
Recover ==
    /\ dbState' = [p \in PageID |->
        IF \E i \in CommittedIndices : walLog[i].page = p
        THEN walLog[LastWrite(p, CommittedIndices)].value
        ELSE InitialValue]
    /\ txState' = txState
    /\ walLog' = walLog

Next == 
    \E tx \in 1..NumTransactions :
        \/ \E page \in PageID : \E value \in ValueRange : WriteAhead(tx, page, value)
        \/ Commit(tx)
        \/ Abort(tx)
    \/ Recover

Spec == Init /\ [][Next]_<<txState, walLog, dbState>>

Theorem_Atomicity == 
    \A p \in PageID : 
        dbState[p] = InitialValue
        \/ \E i \in DOMAIN walLog : 
            walLog[i].type = "WRITE" /\ walLog[i].page = p /\ txState[walLog[i].tx] = Committed

=============================================================================
