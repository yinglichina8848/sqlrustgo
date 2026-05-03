---- MODULE PROOF_016_mvcc_ssi_TTrace_1777766245 ----
EXTENDS Sequences, TLCExt, PROOF_016_mvcc_ssi_TEConstants, Toolbox, Naturals, TLC, PROOF_016_mvcc_ssi

_expression ==
    LET PROOF_016_mvcc_ssi_TEExpression == INSTANCE PROOF_016_mvcc_ssi_TEExpression
    IN PROOF_016_mvcc_ssi_TEExpression!expression
----

_trace ==
    LET PROOF_016_mvcc_ssi_TETrace == INSTANCE PROOF_016_mvcc_ssi_TETrace
    IN PROOF_016_mvcc_ssi_TETrace!trace
----

_inv ==
    ~(
        TLCGet("level") = Len(_TETrace)
        /\
        txnTable = ((t1 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0] @@ t2 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0]))
        /\
        commitQueue = (<<>>)
        /\
        commitLog = (<<[snapTS |-> 0, txnId |-> t1, keys |-> {}]>>)
        /\
        aborted = ({})
        /\
        store = ()
    )
----

_init ==
    /\ store = _TETrace[1].store
    /\ aborted = _TETrace[1].aborted
    /\ txnTable = _TETrace[1].txnTable
    /\ commitQueue = _TETrace[1].commitQueue
    /\ commitLog = _TETrace[1].commitLog
----

_next ==
    /\ \E i,j \in DOMAIN _TETrace:
        /\ \/ /\ j = i + 1
              /\ i = TLCGet("level")
        /\ store  = _TETrace[i].store
        /\ store' = _TETrace[j].store
        /\ aborted  = _TETrace[i].aborted
        /\ aborted' = _TETrace[j].aborted
        /\ txnTable  = _TETrace[i].txnTable
        /\ txnTable' = _TETrace[j].txnTable
        /\ commitQueue  = _TETrace[i].commitQueue
        /\ commitQueue' = _TETrace[j].commitQueue
        /\ commitLog  = _TETrace[i].commitLog
        /\ commitLog' = _TETrace[j].commitLog

\* Uncomment the ASSUME below to write the states of the error trace
\* to the given file in Json format. Note that you can pass any tuple
\* to `JsonSerialize`. For example, a sub-sequence of _TETrace.
    \* ASSUME
    \*     LET J == INSTANCE Json
    \*         IN J!JsonSerialize("PROOF_016_mvcc_ssi_TTrace_1777766245.json", _TETrace)

=============================================================================

 Note that you can extract this module `PROOF_016_mvcc_ssi_TEExpression`
  to a dedicated file to reuse `expression` (the module in the 
  dedicated `PROOF_016_mvcc_ssi_TEExpression.tla` file takes precedence 
  over the module `PROOF_016_mvcc_ssi_TEExpression` below).

---- MODULE PROOF_016_mvcc_ssi_TEExpression ----
EXTENDS Sequences, TLCExt, PROOF_016_mvcc_ssi_TEConstants, Toolbox, Naturals, TLC, PROOF_016_mvcc_ssi

expression == 
    [
        \* To hide variables of the `PROOF_016_mvcc_ssi` spec from the error trace,
        \* remove the variables below.  The trace will be written in the order
        \* of the fields of this record.
        store |-> store
        ,aborted |-> aborted
        ,txnTable |-> txnTable
        ,commitQueue |-> commitQueue
        ,commitLog |-> commitLog
        
        \* Put additional constant-, state-, and action-level expressions here:
        \* ,_stateNumber |-> _TEPosition
        \* ,_storeUnchanged |-> store = store'
        
        \* Format the `store` variable as Json value.
        \* ,_storeJson |->
        \*     LET J == INSTANCE Json
        \*     IN J!ToJson(store)
        
        \* Lastly, you may build expressions over arbitrary sets of states by
        \* leveraging the _TETrace operator.  For example, this is how to
        \* count the number of times a spec variable changed up to the current
        \* state in the trace.
        \* ,_storeModCount |->
        \*     LET F[s \in DOMAIN _TETrace] ==
        \*         IF s = 1 THEN 0
        \*         ELSE IF _TETrace[s].store # _TETrace[s-1].store
        \*             THEN 1 + F[s-1] ELSE F[s-1]
        \*     IN F[_TEPosition - 1]
    ]

=============================================================================



Parsing and semantic processing can take forever if the trace below is long.
 In this case, it is advised to uncomment the module below to deserialize the
 trace from a generated binary file.

\*
\*---- MODULE PROOF_016_mvcc_ssi_TETrace ----
\*EXTENDS IOUtils, PROOF_016_mvcc_ssi_TEConstants, TLC, PROOF_016_mvcc_ssi
\*
\*trace == IODeserialize("PROOF_016_mvcc_ssi_TTrace_1777766245.bin", TRUE)
\*
\*=============================================================================
\*

---- MODULE PROOF_016_mvcc_ssi_TETrace ----
EXTENDS PROOF_016_mvcc_ssi_TEConstants, TLC, PROOF_016_mvcc_ssi

trace == 
    <<
    ([txnTable |-> (t1 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0] @@ t2 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0]),commitQueue |-> <<>>,commitLog |-> <<>>,aborted |-> {},store |-> (k1 :> 0 @@ k2 :> 0)]),
    ([txnTable |-> (t1 :> [status |-> "active", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0] @@ t2 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0]),commitQueue |-> <<>>,commitLog |-> <<>>,aborted |-> {},store |-> (k1 :> 0 @@ k2 :> 0)]),
    ([txnTable |-> (t1 :> [status |-> "committing", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0] @@ t2 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0]),commitQueue |-> <<t1>>,commitLog |-> <<>>,aborted |-> {},store |-> (k1 :> 0 @@ k2 :> 0)]),
    ([txnTable |-> (t1 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0] @@ t2 :> [status |-> "idle", readSet |-> {}, writeSet |-> {}, snapTS |-> 0, startTS |-> 0]),commitQueue |-> <<>>,commitLog |-> <<[snapTS |-> 0, txnId |-> t1, keys |-> {}]>>,aborted |-> {},store |-> ])
    >>
----


=============================================================================

---- MODULE PROOF_016_mvcc_ssi_TEConstants ----
EXTENDS PROOF_016_mvcc_ssi

CONSTANTS null, t1, t2, k1, k2

=============================================================================

---- CONFIG PROOF_016_mvcc_ssi_TTrace_1777766245 ----
CONSTANTS
    NULL = null
    TxnId = { t1 , t2 }
    Key = { k1 , k2 }
    k1 = k1
    k2 = k2
    null = null
    t1 = t1
    t2 = t2

INVARIANT
    _inv

CHECK_DEADLOCK
    \* CHECK_DEADLOCK off because of PROPERTY or INVARIANT above.
    FALSE

INIT
    _init

NEXT
    _next

CONSTANT
    _TETrace <- _trace

ALIAS
    _expression
=============================================================================
\* Generated on Sun May 03 07:57:26 CST 2026