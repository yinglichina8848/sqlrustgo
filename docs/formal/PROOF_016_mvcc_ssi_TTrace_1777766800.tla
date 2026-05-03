---- MODULE PROOF_016_mvcc_ssi_TTrace_1777766800 ----
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
        txnStatus = ((t1 :> "active" @@ t2 :> "active"))
        /\
        commitLog = (<<>>)
        /\
        aborted = ({})
        /\
        txnReadSet = ((t1 :> {t1} @@ t2 :> {}))
        /\
        step = (11)
        /\
        store = ((t1 :> 0 @@ t2 :> 0))
        /\
        txnWriteSet = ((t1 :> {} @@ t2 :> {}))
    )
----

_init ==
    /\ store = _TETrace[1].store
    /\ txnReadSet = _TETrace[1].txnReadSet
    /\ txnWriteSet = _TETrace[1].txnWriteSet
    /\ aborted = _TETrace[1].aborted
    /\ step = _TETrace[1].step
    /\ txnStatus = _TETrace[1].txnStatus
    /\ commitLog = _TETrace[1].commitLog
----

_next ==
    /\ \E i,j \in DOMAIN _TETrace:
        /\ \/ /\ j = i + 1
              /\ i = TLCGet("level")
        /\ store  = _TETrace[i].store
        /\ store' = _TETrace[j].store
        /\ txnReadSet  = _TETrace[i].txnReadSet
        /\ txnReadSet' = _TETrace[j].txnReadSet
        /\ txnWriteSet  = _TETrace[i].txnWriteSet
        /\ txnWriteSet' = _TETrace[j].txnWriteSet
        /\ aborted  = _TETrace[i].aborted
        /\ aborted' = _TETrace[j].aborted
        /\ step  = _TETrace[i].step
        /\ step' = _TETrace[j].step
        /\ txnStatus  = _TETrace[i].txnStatus
        /\ txnStatus' = _TETrace[j].txnStatus
        /\ commitLog  = _TETrace[i].commitLog
        /\ commitLog' = _TETrace[j].commitLog

\* Uncomment the ASSUME below to write the states of the error trace
\* to the given file in Json format. Note that you can pass any tuple
\* to `JsonSerialize`. For example, a sub-sequence of _TETrace.
    \* ASSUME
    \*     LET J == INSTANCE Json
    \*         IN J!JsonSerialize("PROOF_016_mvcc_ssi_TTrace_1777766800.json", _TETrace)

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
        ,txnReadSet |-> txnReadSet
        ,txnWriteSet |-> txnWriteSet
        ,aborted |-> aborted
        ,step |-> step
        ,txnStatus |-> txnStatus
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
\*trace == IODeserialize("PROOF_016_mvcc_ssi_TTrace_1777766800.bin", TRUE)
\*
\*=============================================================================
\*

---- MODULE PROOF_016_mvcc_ssi_TETrace ----
EXTENDS PROOF_016_mvcc_ssi_TEConstants, TLC, PROOF_016_mvcc_ssi

trace == 
    <<
    ([txnStatus |-> (t1 :> "idle" @@ t2 :> "idle"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {} @@ t2 :> {}),step |-> 0,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "idle"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {} @@ t2 :> {}),step |-> 1,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {} @@ t2 :> {}),step |-> 2,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 3,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 4,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 5,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 6,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 7,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 8,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 9,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 10,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})]),
    ([txnStatus |-> (t1 :> "active" @@ t2 :> "active"),commitLog |-> <<>>,aborted |-> {},txnReadSet |-> (t1 :> {t1} @@ t2 :> {}),step |-> 11,store |-> (t1 :> 0 @@ t2 :> 0),txnWriteSet |-> (t1 :> {} @@ t2 :> {})])
    >>
----


=============================================================================

---- MODULE PROOF_016_mvcc_ssi_TEConstants ----
EXTENDS PROOF_016_mvcc_ssi

CONSTANTS null, t1, t2

=============================================================================

---- CONFIG PROOF_016_mvcc_ssi_TTrace_1777766800 ----
CONSTANTS
    NULL = null
    Tbl = { t1 , t2 }
    t1 = t1
    t2 = t2
    null = null

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
\* Generated on Sun May 03 08:06:42 CST 2026