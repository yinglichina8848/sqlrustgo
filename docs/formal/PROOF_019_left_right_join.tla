-------------------------------- MODULE PROOF_019_left_right_join --------------------------------
(* PROOF-019: LEFT/RIGHT OUTER JOIN Algorithm Verification *)
(* Tool: TLA+ / TLC Model Checker *)

EXTENDS Integers, FiniteSets, Sequences

VARIABLES left_buffer, right_buffer, left_result, right_result, null_marker

NullMarker == 0

Init ==
    /\ left_buffer = {1, 2}
    /\ right_buffer = {1}
    /\ left_result = <<>>
    /\ right_result = <<>>
    /\ null_marker = NullMarker

LeftJoinStep ==
    IF left_buffer = {} THEN
        UNCHANGED <<left_buffer, right_buffer, left_result, right_result, null_marker>>
    ELSE
        LET id == CHOOSE x \in left_buffer : TRUE IN
        LET matched == IF id \in right_buffer THEN id ELSE null_marker IN
        /\ left_result' = Append(left_result, [id |-> id, matched |-> matched, side |-> "L"])
        /\ left_buffer' = left_buffer \ {id}
        /\ UNCHANGED <<right_buffer, right_result, null_marker>>

RightJoinStep ==
    IF right_buffer = {} THEN
        UNCHANGED <<left_buffer, right_buffer, left_result, right_result, null_marker>>
    ELSE
        LET id == CHOOSE x \in right_buffer : TRUE IN
        LET matched == IF id \in left_buffer THEN id ELSE null_marker IN
        /\ right_result' = Append(right_result, [id |-> id, matched |-> matched, side |-> "R"])
        /\ right_buffer' = right_buffer \ {id}
        /\ UNCHANGED <<left_buffer, left_result, null_marker>>

Next ==
    \/ LeftJoinStep
    \/ RightJoinStep

vars == <<left_buffer, right_buffer, left_result, right_result, null_marker>>

OuterJoinSpec == Init /\ [][Next]_vars /\ WF_vars(Next)

TypeInvariant ==
    /\ null_marker = NullMarker
    /\ \A r \in left_result : r.side = "L"
    /\ \A r \in right_result : r.side = "R"

LeftResultComplete == (left_buffer = {}) => (Len(left_result) = 2)
RightResultComplete == (right_buffer = {}) => (Len(right_result) = 1)
================================================================================
