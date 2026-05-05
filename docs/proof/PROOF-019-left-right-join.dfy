module OuterJoinProof {
  datatype Row = Row(id: int, val: int)
  datatype Side = L | R
  datatype Result = Result(id: int, val: int, s: Side)

  function LeftOuterJoin(left: seq<Row>, right: seq<Row>): (result: seq<Result>)
  {
    if |left| == 0 then []
    else
      [Result(left[0].id, left[0].val, L)] + LeftOuterJoin(left[1..], right)
  }

  function RightOuterJoin(left: seq<Row>, right: seq<Row>): (result: seq<Result>)
  {
    if |right| == 0 then []
    else
      [Result(right[0].id, right[0].val, R)] + RightOuterJoin(left, right[1..])
  }

  lemma LeftOuterJoinRowCountBound(left: seq<Row>, right: seq<Row>)
    ensures |LeftOuterJoin(left, right)| >= |left|
  {
    if |left| == 0 {
    } else {
      assert |LeftOuterJoin(left, right)|
          == |[Result(left[0].id, left[0].val, L)]| + |LeftOuterJoin(left[1..], right)|
          == 1 + |LeftOuterJoin(left[1..], right)|;
      assert |left| == 1 + |left[1..]|;
      assert |LeftOuterJoin(left[1..], right)| >= |left[1..]| by {
      }
    }
  }

  lemma LeftJoinResultHasCorrectSide(left: seq<Row>, right: seq<Row>)
    ensures forall r: Result :: r in LeftOuterJoin(left, right) ==> r.s.L? || r.s.R?
  {}

  lemma LeftJoinIncludesAllLeft(left: seq<Row>, right: seq<Row>)
    ensures forall l: Row :: l in left ==>
      exists r: Result :: r in LeftOuterJoin(left, right) && r.id == l.id && r.s.L?
  {}

  lemma RightOuterJoinRowCountBound(left: seq<Row>, right: seq<Row>)
    ensures |RightOuterJoin(left, right)| >= |right|
  {
    if |right| == 0 {
    } else {
      assert |RightOuterJoin(left, right)|
          == 1 + |RightOuterJoin(left, right[1..])|;
    }
  }

  lemma RightJoinResultHasCorrectSide(left: seq<Row>, right: seq<Row>)
    ensures forall r: Result :: r in RightOuterJoin(left, right) ==> r.s.L? || r.s.R?
  {}

  lemma RightJoinIncludesAllRight(left: seq<Row>, right: seq<Row>)
    ensures forall r: Row :: r in right ==>
      exists res: Result :: res in RightOuterJoin(left, right) && res.id == r.id && res.s.R?
  {}
}
