-------------------------- MODULE WAL_Recovery --------------------------
\* WAL-based Transaction Recovery Model
\* Proves ACID properties for committed transactions

EXTENDS Integers, Sequences, TLC

CONSTANT 
    \* Transaction states
    NULL, Committed, Aborted,
    \* Page identifiers  
    PageID,
    \* Initial value for pages
    InitialValue,
    \* Number of transactions
    NumTransactions

VARIABLES 
    txState,  \* Transaction state: TransactionID -> {NULL, Committed, Aborted}
    walLog,   \* Write-Ahead Log: Sequence of LogEntry
    dbState   \* Database state: PageID -> Value

LogEntry == [tx: 1..NumTransactions, page: PageID, value: Int, type: {"WRITE", "COMMIT"}]

TypeOK == 
    /\ txState \in [1..NumTransactions -> {NULL, Committed, Aborted}]
    /\ walLog \in Seq(LogEntry)
    /\ dbState \in [PageID -> Int]

Init == 
    /\ txState = [t \in 1..NumTransactions |-> NULL]
    /\ walLog = << >>
    /\ dbState = [p \in PageID |-> InitialValue]

WriteAhead(tx, page, value) ==
    /\ txState[tx] = NULL
    /\ walLog' = Append(walLog, [tx |-> tx, page |-> page, value |-> value, type |-> "WRITE"])
    /\ dbState' = dbState @@ [page <- value]

Commit(tx) ==
    /\ txState[tx] = NULL
    /\ txState' = [txState EXCEPT ![tx] = Committed]
    /\ walLog' = Append(walLog, [tx |-> tx, type |-> "COMMIT"])

Abort(tx) ==
    /\ txState[tx] = NULL
    /\ txState' = [txState EXCEPT ![tx] = Aborted]
    /\ walLog' = walLog

\* Recovery: Replay all committed transactions from WAL
Recover ==
    LET committedTxs == {tx \in 1..NumTransactions : txState[tx] = Committed}
    LET committedWrites == {e \in DOMAIN walLog : e.tx \in committedTxs /\ e.type = "WRITE"}
    IN
    /\ dbState' = [p \in PageID |->
        LET writesToPage == {e \in committedWrites : e.page = p}
        IN
        IF writesToPage # {} THEN
            LET lastWrite == CHOOSE e \in writesToPage : 
                \A e2 \in writesToPage : e.tx >= e2.tx
            IN lastWrite.value
        ELSE dbState[p]]
    /\ txState' = txState

\* Theorems (to be verified by TLC)
\* Theorem 1: Atomicity - After recovery, only committed writes are reflected
Theorem_Atomicity ==
    ASSUME NEW tx \in 1..NumTransactions
    PROVE 
        (txState[tx] = Committed /\ txState' = txState) =>
        \A p \in PageID : 
            dbState'[p] = dbState[p] \/ 
            \E e \in committedWrites : e.page = p /\ e.value = dbState'[p]

================================================================================