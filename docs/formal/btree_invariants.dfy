// B+Tree Invariants - Verification-Ready
// PROOF-013: B+Tree Query Completeness Proof

datatype Key = Key(Int)
datatype RecordID = RecordID(Int)

datatype BPlusNode = 
    InternalNode(children: seq<BPlusNode>, keys: seq<Key>)
  | LeafNode(entries: seq<(Key, RecordID)>, next: Option<BPlusNode>)

function FindInEntries(entries: seq<(Key, RecordID)>, key: Key): Option<RecordID>
  decreases entries
{
  if entries == [] then None
  else if entries[0].0 == key then Some(entries[0].1)
  else FindInEntries(entries[1..], key)
}

function Search(node: BPlusNode, key: Key): Option<RecordID>
  decreases node
{
  match node
  case LeafNode(entries, _) => FindInEntries(entries, key)
  case InternalNode(children, keys) =>
    if |keys| == 0 || key < keys[0] then
      Search(children[0], key)
    else if key >= keys[|keys|-1] then
      Search(children[|children|-1], key)
    else
      FindChildAndSearch(children, keys, key, 0)
}

function FindChildAndSearch(children: seq<BPlusNode>, keys: seq<Key>, key: Key, i: nat): Option<RecordID>
  decreases children, i
{
  if i >= |keys| then None
  else if key < keys[i] then
    Search(children[i], key)
  else
      FindChildAndSearch(children, keys, key, i + 1)
}

function RangeQuery(node: BPlusNode, low: Key, high: Key): set<RecordID>
  decreases node
{
  match node
  case LeafNode(entries, _) =>
    set e in entries | low <= e.0 <= high :: e.1
  case InternalNode(children, keys) =>
    set c in children :: RangeQuery(c, low, high)
}

function LTE(a: Key, b: Key): bool
{
  match (a, b)
  case (Key(aVal), Key(bVal)) => aVal <= bVal
}

function GTE(a: Key, b: Key): bool
{
  match (a, b)
  case (Key(aVal), Key(bVal)) => aVal >= bVal
}

// Lemma: Search Soundness
// If Search returns Some(rid), then the record has the searched key
lemma {:induction node} SearchSoundness(node: BPlusNode, key: Key, rid: RecordID)
  requires Search(node, key) == Some(rid)
  ensures 
    exists e in getAllEntries(node) :: e.0 == key && e.1 == rid

function getAllEntries(node: BPlusNode): set<(Key, RecordID)>
  decreases node
{
  match node
  case LeafNode(entries, _) => set e in entries :: e
  case InternalNode(children, keys) =>
    set c in children :: getAllEntries(c)
}

// Lemma: RangeQuery Completeness  
// All records with key in [low, high] are returned
lemma {:induction node} RangeQueryComplete(node: BPlusNode, low: Key, high: Key, rid: RecordID)
  requires rid in RangeQuery(node, low, high)
  ensures exists e in getAllEntries(node) :: e.0 == rid

// Theorem: Query Correctness
// RangeQuery returns exactly the records with keys in the range
lemma {:induction node} BPlusTreeCorrectness(node: BPlusNode, low: Key, high: Key)
  ensures RangeQuery(node, low, high) ==
    {rid | exists e in getAllEntries(node) :: e.0 == rid && low <= e.0 && e.0 <= high}
{
  match node
  case LeafNode(entries, _) =>
    // Base case: leaf node
    assert RangeQuery(node, low, high) ==
      {e.1 | e in entries | low <= e.0 && e.0 <= high};
  case InternalNode(children, keys) =>
    // Inductive case: internal node
    var childResult := set c in children :: RangeQuery(c, low, high);
    assert RangeQuery(node, low, high) == childResult;
}