--------------------------- MODULE PROOF_015_ddl_atomicity ---------------------------
EXTENDS Sequences, FiniteSets, Integers, TLC

CONSTANTS
  NULL,
  Tbl

VARIABLES
  schema,
  ddlLog,
  ddlInProgress,
  ddlTarget,
  txnState,
  catalogBuffer,
  systemReady

TypeInvariant ==
  /\ schema \subseteq Tbl
  /\ ddlLog \in Seq([op: STRING, target: Tbl])
  /\ ddlInProgress \in {"idle", "create", "drop", "index"}
  /\ ddlTarget \in Tbl \cup {NULL}
  /\ txnState \in {"idle", "active", "committing", "rolled_back", "committed"}
  /\ catalogBuffer \subseteq Tbl
  /\ systemReady \in BOOLEAN

Init ==
  /\ schema = {}
  /\ ddlLog = << >>
  /\ ddlInProgress = "idle"
  /\ ddlTarget = NULL
  /\ txnState = "idle"
  /\ catalogBuffer = {}
  /\ systemReady = TRUE

DoCreateTable(tbl) ==
  /\ txnState = "idle"
  /\ systemReady = TRUE
  /\ tbl \in Tbl
  /\ tbl \notin schema
  /\ ddlInProgress' = "create"
  /\ ddlTarget' = tbl
  /\ catalogBuffer' = catalogBuffer \cup {tbl}
  /\ txnState' = "active"
  /\ schema' = schema
  /\ ddlLog' = ddlLog
  /\ systemReady' = systemReady

DoDropTable(tbl) ==
  /\ txnState = "idle"
  /\ systemReady = TRUE
  /\ tbl \in Tbl
  /\ tbl \in schema
  /\ ddlInProgress' = "drop"
  /\ ddlTarget' = tbl
  /\ catalogBuffer' = catalogBuffer \ {tbl}
  /\ txnState' = "active"
  /\ schema' = schema
  /\ ddlLog' = ddlLog
  /\ systemReady' = systemReady

DoCreateIndex(tbl) ==
  /\ txnState = "idle"
  /\ systemReady = TRUE
  /\ tbl \in Tbl
  /\ tbl \in schema
  /\ ddlInProgress' = "index"
  /\ ddlTarget' = tbl
  /\ txnState' = "active"
  /\ catalogBuffer' = catalogBuffer
  /\ schema' = schema
  /\ ddlLog' = ddlLog
  /\ systemReady' = systemReady

CommitDDL ==
  /\ txnState = "active"
  /\ ddlInProgress /= "idle"
  /\ txnState' = "committing"
  /\ IF ddlInProgress = "create"
     THEN schema' = schema \cup catalogBuffer
     ELSE IF ddlInProgress = "drop"
          THEN schema' = schema \ {ddlTarget}
          ELSE schema' = schema
  /\ ddlLog' = Append(ddlLog, [op |-> ddlInProgress, target |-> ddlTarget])
  /\ ddlInProgress' = "idle"
  /\ ddlTarget' = NULL
  /\ catalogBuffer' = {}
  /\ txnState' = "committed"
  /\ systemReady' = systemReady

RollbackDDL ==
  /\ txnState = "active"
  /\ ddlInProgress /= "idle"
  /\ catalogBuffer' = {}
  /\ ddlInProgress' = "idle"
  /\ ddlTarget' = NULL
  /\ txnState' = "rolled_back"
  /\ schema' = schema
  /\ ddlLog' = ddlLog
  /\ systemReady' = systemReady

ResetToIdle ==
  /\ txnState \in {"committed", "rolled_back"}
  /\ txnState' = "idle"
  /\ ddlInProgress' = ddlInProgress
  /\ ddlTarget' = ddlTarget
  /\ catalogBuffer' = catalogBuffer
  /\ schema' = schema
  /\ ddlLog' = ddlLog
  /\ systemReady' = systemReady

SystemFailure ==
  /\ ddlInProgress /= "idle"
  /\ systemReady' = FALSE
  /\ txnState' = "rolled_back"
  /\ ddlInProgress' = "idle"
  /\ ddlTarget' = NULL
  /\ catalogBuffer' = {}
  /\ schema' = schema
  /\ ddlLog' = ddlLog

SystemRecovery ==
  /\ systemReady = FALSE
  /\ systemReady' = TRUE
  /\ txnState' = "idle"
  /\ ddlInProgress' = ddlInProgress
  /\ ddlTarget' = ddlTarget
  /\ catalogBuffer' = catalogBuffer
  /\ schema' = schema
  /\ ddlLog' = ddlLog

Next ==
  \/ \E tbl \in Tbl : DoCreateTable(tbl)
  \/ \E tbl \in Tbl : DoDropTable(tbl)
  \/ \E tbl \in Tbl : DoCreateIndex(tbl)
  \/ CommitDDL
  \/ RollbackDDL
  \/ ResetToIdle
  \/ SystemFailure
  \/ SystemRecovery

Spec == Init /\ [][Next]_<<schema, ddlLog, ddlInProgress, ddlTarget, txnState, catalogBuffer, systemReady>>

AllOrNothing ==
  /\ txnState = "committed" => catalogBuffer = {}
  /\ txnState = "rolled_back" => catalogBuffer = {}

SchemaConsistency ==
  \A i \in 1..Len(ddlLog) :
    LET entry == ddlLog[i] IN
    \/ entry.op = "create" => entry.target \in schema
    \/ entry.op = "drop"   => entry.target \notin schema
    \/ entry.op = "index"  => entry.target \in schema

CommitImpliesLog ==
  \A tbl \in Tbl :
    tbl \in schema =>
      \E i \in 1..Len(ddlLog) :
        \/ ddlLog[i].op = "create" /\ ddlLog[i].target = tbl
        \/ ddlLog[i].op = "index" /\ ddlLog[i].target = tbl

RollbackClearsBuffer ==
  txnState = "rolled_back" => catalogBuffer = {}

NoPartialDDL ==
  txnState = "committed" => catalogBuffer = {}

=============================================================================
