---- MODULE PROOF_015_ddl_atomicity_TTrace_1777766133 ----
EXTENDS Sequences, TLCExt, Toolbox, PROOF_015_ddl_atomicity_TEConstants, Naturals, TLC, PROOF_015_ddl_atomicity

_expression ==
    LET PROOF_015_ddl_atomicity_TEExpression == INSTANCE PROOF_015_ddl_atomicity_TEExpression
    IN PROOF_015_ddl_atomicity_TEExpression!expression
----

_trace ==
    LET PROOF_015_ddl_atomicity_TETrace == INSTANCE PROOF_015_ddl_atomicity_TETrace
    IN PROOF_015_ddl_atomicity_TETrace!trace
----

_inv ==
    ~(
        TLCGet("level") = Len(_TETrace)
        /\
        schema = ({})
        /\
        ddlLog = (<<>>)
        /\
        systemReady = ()
        /\
        txnState = ("active")
        /\
        ddlTarget = (t1)
        /\
        ddlInProgress = ("create")
        /\
        catalogBuffer = ({t1})
    )
----

_init ==
    /\ catalogBuffer = _TETrace[1].catalogBuffer
    /\ ddlTarget = _TETrace[1].ddlTarget
    /\ txnState = _TETrace[1].txnState
    /\ systemReady = _TETrace[1].systemReady
    /\ schema = _TETrace[1].schema
    /\ ddlLog = _TETrace[1].ddlLog
    /\ ddlInProgress = _TETrace[1].ddlInProgress
----

_next ==
    /\ \E i,j \in DOMAIN _TETrace:
        /\ \/ /\ j = i + 1
              /\ i = TLCGet("level")
        /\ catalogBuffer  = _TETrace[i].catalogBuffer
        /\ catalogBuffer' = _TETrace[j].catalogBuffer
        /\ ddlTarget  = _TETrace[i].ddlTarget
        /\ ddlTarget' = _TETrace[j].ddlTarget
        /\ txnState  = _TETrace[i].txnState
        /\ txnState' = _TETrace[j].txnState
        /\ systemReady  = _TETrace[i].systemReady
        /\ systemReady' = _TETrace[j].systemReady
        /\ schema  = _TETrace[i].schema
        /\ schema' = _TETrace[j].schema
        /\ ddlLog  = _TETrace[i].ddlLog
        /\ ddlLog' = _TETrace[j].ddlLog
        /\ ddlInProgress  = _TETrace[i].ddlInProgress
        /\ ddlInProgress' = _TETrace[j].ddlInProgress

\* Uncomment the ASSUME below to write the states of the error trace
\* to the given file in Json format. Note that you can pass any tuple
\* to `JsonSerialize`. For example, a sub-sequence of _TETrace.
    \* ASSUME
    \*     LET J == INSTANCE Json
    \*         IN J!JsonSerialize("PROOF_015_ddl_atomicity_TTrace_1777766133.json", _TETrace)

=============================================================================

 Note that you can extract this module `PROOF_015_ddl_atomicity_TEExpression`
  to a dedicated file to reuse `expression` (the module in the 
  dedicated `PROOF_015_ddl_atomicity_TEExpression.tla` file takes precedence 
  over the module `PROOF_015_ddl_atomicity_TEExpression` below).

---- MODULE PROOF_015_ddl_atomicity_TEExpression ----
EXTENDS Sequences, TLCExt, Toolbox, PROOF_015_ddl_atomicity_TEConstants, Naturals, TLC, PROOF_015_ddl_atomicity

expression == 
    [
        \* To hide variables of the `PROOF_015_ddl_atomicity` spec from the error trace,
        \* remove the variables below.  The trace will be written in the order
        \* of the fields of this record.
        catalogBuffer |-> catalogBuffer
        ,ddlTarget |-> ddlTarget
        ,txnState |-> txnState
        ,systemReady |-> systemReady
        ,schema |-> schema
        ,ddlLog |-> ddlLog
        ,ddlInProgress |-> ddlInProgress
        
        \* Put additional constant-, state-, and action-level expressions here:
        \* ,_stateNumber |-> _TEPosition
        \* ,_catalogBufferUnchanged |-> catalogBuffer = catalogBuffer'
        
        \* Format the `catalogBuffer` variable as Json value.
        \* ,_catalogBufferJson |->
        \*     LET J == INSTANCE Json
        \*     IN J!ToJson(catalogBuffer)
        
        \* Lastly, you may build expressions over arbitrary sets of states by
        \* leveraging the _TETrace operator.  For example, this is how to
        \* count the number of times a spec variable changed up to the current
        \* state in the trace.
        \* ,_catalogBufferModCount |->
        \*     LET F[s \in DOMAIN _TETrace] ==
        \*         IF s = 1 THEN 0
        \*         ELSE IF _TETrace[s].catalogBuffer # _TETrace[s-1].catalogBuffer
        \*             THEN 1 + F[s-1] ELSE F[s-1]
        \*     IN F[_TEPosition - 1]
    ]

=============================================================================



Parsing and semantic processing can take forever if the trace below is long.
 In this case, it is advised to uncomment the module below to deserialize the
 trace from a generated binary file.

\*
\*---- MODULE PROOF_015_ddl_atomicity_TETrace ----
\*EXTENDS IOUtils, PROOF_015_ddl_atomicity_TEConstants, TLC, PROOF_015_ddl_atomicity
\*
\*trace == IODeserialize("PROOF_015_ddl_atomicity_TTrace_1777766133.bin", TRUE)
\*
\*=============================================================================
\*

---- MODULE PROOF_015_ddl_atomicity_TETrace ----
EXTENDS PROOF_015_ddl_atomicity_TEConstants, TLC, PROOF_015_ddl_atomicity

trace == 
    <<
    ([schema |-> {},ddlLog |-> <<>>,systemReady |-> TRUE,txnState |-> "idle",ddlTarget |-> null,ddlInProgress |-> "idle",catalogBuffer |-> {}]),
    ([schema |-> {},ddlLog |-> <<>>,systemReady |-> ,txnState |-> "active",ddlTarget |-> t1,ddlInProgress |-> "create",catalogBuffer |-> {t1}])
    >>
----


=============================================================================

---- MODULE PROOF_015_ddl_atomicity_TEConstants ----
EXTENDS PROOF_015_ddl_atomicity

CONSTANTS null, t1, t2

=============================================================================

---- CONFIG PROOF_015_ddl_atomicity_TTrace_1777766133 ----
CONSTANTS
    NULL = null
    Tbl = { t1 , t2 }
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
\* Generated on Sun May 03 07:55:34 CST 2026