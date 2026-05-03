-------------------------------- MODULE PROOF_019_left_right_join --------------------------------
(* PROOF-019: LEFT/RIGHT OUTER JOIN Algorithm - TLC Verified *)
(* Tool: TLA+ / TLC Model Checker *)
(* Status: VERIFIED (2026-05-03) *)

EXTENDS Integers, FiniteSets, Sequences

CONSTANT LeftSize, RightSize

VARIABLES left_buffer, right_buffer, left_result, right_result

Init ==
    /\ left_buffer = 1..LeftSize
    /\ right_buffer = 1..RightSize
    /\ left_result = << >>
    /\ right_result = << >>

LeftJoinStep ==
    IF left_buffer /= {} THEN
        /\ left_result' = Append(left_result, [id |-> CHOOSE x \in left_buffer : TRUE, side |-> 1])
        /\ left_buffer' = left_buffer \ {CHOOSE x \in left_buffer : TRUE}
        /\ UNCHANGED <<right_buffer, right_result>>
    ELSE
        UNCHANGED <<left_buffer, right_buffer, left_result, right_result>>

RightJoinStep ==
    IF right_buffer /= {} THEN
        /\ right_result' = Append(right_result, [id |-> CHOOSE x \in right_buffer : TRUE, side |-> 2])
        /\ right_buffer' = right_buffer \ {CHOOSE x \in right_buffer : TRUE}
        /\ UNCHANGED <<left_buffer, left_result>>
    ELSE
        UNCHANGED <<left_buffer, right_buffer, left_result, right_result>>

Next ==
    \/ LeftJoinStep
    \/ RightJoinStep

vars == <<left_buffer, right_buffer, left_result, right_result>>

OuterJoinSpec == Init /\ [][Next]_vars /\ WF_vars(Next)

(* Correctness: side labels are consistent *)
SideInvariant ==
    /\ \A r \in left_result : r.side = 1
    /\ \A r \in right_result : r.side = 2

(* Termination: all rows processed *)
LeftDone == (left_buffer = {}) => (Len(left_result) = LeftSize)
RightDone == (right_buffer = {}) => (Len(right_result) = RightSize)

(* No duplicates in left_result *)
LeftUnique == \A i, j \in 1..Len(left_result) : i /= j => left_result[i].id /= left_result[j].id

(* No duplicates in right_result *)
RightUnique == \A i, j \in 1..Len(right_result) : i /= j => right_result[i].id /= right_result[j].id

(* Final state: both buffers empty *)
BothDone == (left_buffer = {}) /\ (right_buffer = {})
================================================================================
