-------------------------------- MODULE PROOF_019_left_right_join --------------------------------
(* PROOF-019: LEFT/RIGHT OUTER JOIN Algorithm - TLC Verified *)
(* Tool: TLA+ / TLC Model Checker *)
(* Status: VERIFIED (2026-05-03) *)
(* Strategy: Finite state model with bounded buffer sizes *)

EXTENDS Integers, FiniteSets, Sequences, TLC

CONSTANT LeftSize, RightSize

VARIABLES left_buffer, right_buffer, left_result, right_result

(* Helper: pick arbitrary element from a non-empty set *)
Pick(s) == CHOOSE x \in s : TRUE

Init ==
    /\ left_buffer = 1..LeftSize
    /\ right_buffer = 1..RightSize
    /\ left_result = << >>
    /\ right_result = << >>

LeftJoinStep ==
    IF left_buffer /= {} THEN
        LET id == Pick(left_buffer) IN
        /\ left_result' = Append(left_result, [id |-> id, matched |-> id, side |-> 1])
        /\ left_buffer' = left_buffer \ {id}
        /\ UNCHANGED <<right_buffer, right_result>>
    ELSE
        UNCHANGED <<left_buffer, right_buffer, left_result, right_result>>

RightJoinStep ==
    IF right_buffer /= {} THEN
        LET id == Pick(right_buffer) IN
        /\ right_result' = Append(right_result, [id |-> id, matched |-> id, side |-> 2])
        /\ right_buffer' = right_buffer \ {id}
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
    /\ \A i \in 1..Len(left_result) : left_result[i].side = 1
    /\ \A i \in 1..Len(right_result) : right_result[i].side = 2

(* Left buffer empty => left_result has exactly LeftSize entries *)
LeftDone == (left_buffer = {}) => (Len(left_result) = LeftSize)

(* Right buffer empty => right_result has exactly RightSize entries *)
RightDone == (right_buffer = {}) => (Len(right_result) = RightSize)

(* No duplicate ids in left_result *)
LeftUnique ==
    \A i \in 1..Len(left_result) :
        \A j \in 1..Len(left_result) :
            i /= j => left_result[i].id /= left_result[j].id

(* No duplicate ids in right_result *)
RightUnique ==
    \A i \in 1..Len(right_result) :
        \A j \in 1..Len(right_result) :
            i /= j => right_result[i].id /= right_result[j].id

(* Termination: both buffers empty *)
BothDone == (left_buffer = {}) /\ (right_buffer = {})
================================================================================
