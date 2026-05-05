--------------------------- MODULE PROOF_023_deadlock_v4 ---------------------------
(* PROOF-023 v4: Multi-Resource Wait-For Graph *)

EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS T1, T2, T3, K1, K2

VARIABLES txnHoldLocks, txnWaitFor

AllTxns == {T1, T2, T3}
AllKeys == {K1, K2}

Holder(k) == { t \in AllTxns : k \in txnHoldLocks[t] }

RECURSIVE Reachable(_,_)
Reachable(t1, t2) ==
  \/ t2 \in txnWaitFor[t1]
  \/ \E t \in txnWaitFor[t1] : /\ t # t1 /\ Reachable(t, t2)

NoSelfWait == \A t \in AllTxns : t \notin txnWaitFor[t]
NoCycle == \A t \in AllTxns : ~Reachable(t, t)

(* If t waits for h, h must hold something useful for t (or have died/waited) *)
WaitConsistent ==
  \A t \in AllTxns :
    \A h \in txnWaitFor[t] :
      \/ \E k \in AllKeys : k \in txnHoldLocks[h]
      \/ h \in txnWaitFor[h]       (* h is itself waiting (chain) *)

Init ==
  /\ txnHoldLocks = [t \in AllTxns |-> {}]
  /\ txnWaitFor = [t \in AllTxns |-> {}]

Acquire(t, k) ==
  /\ k \notin txnHoldLocks[t]
  /\ Holder(k) = {}
  /\ txnHoldLocks' = [txnHoldLocks EXCEPT ![t] = txnHoldLocks[t] \cup {k}]
  /\ UNCHANGED txnWaitFor

Wait(t, k) ==
  LET holders == Holder(k) \ {t} IN
  /\ holders # {}
  /\ txnWaitFor[t] = {}
  /\ \A h \in holders : ~Reachable(h, t)
  /\ txnWaitFor' = [txnWaitFor EXCEPT ![t] = holders]
  /\ UNCHANGED txnHoldLocks

(* Release: clear locks AND everyone waiting on me is unblocked *)
Release(t) ==
  /\ txnHoldLocks[t] # {}
  /\ txnHoldLocks' = [txnHoldLocks EXCEPT ![t] = {}]
  /\ txnWaitFor' = [x \in AllTxns |->
                      IF t \in txnWaitFor[x]
                         THEN txnWaitFor[x] \ {t}
                         ELSE txnWaitFor[x]]

(* Abort: same as Release — all waiters of t are unblocked *)
Abort(t) ==
  /\ txnHoldLocks[t] # {}
  /\ txnHoldLocks' = [txnHoldLocks EXCEPT ![t] = {}]
  /\ txnWaitFor' = [x \in AllTxns |->
                      IF t \in txnWaitFor[x]
                         THEN txnWaitFor[x] \ {t}
                         ELSE txnWaitFor[x]]

Next ==
  \/ \E t \in AllTxns, k \in AllKeys : Acquire(t, k)
  \/ \E t \in AllTxns, k \in AllKeys : Wait(t, k)
  \/ \E t \in AllTxns : Release(t)
  \/ \E t \in AllTxns : Abort(t)

Spec == Init /\ [][Next]_<<txnHoldLocks, txnWaitFor>>

=============================================================================
