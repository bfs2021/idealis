module SequenceFold {
  var total: int
  action init = total' = List(3, 1, 2).foldl(0, (acc, item) => acc * 10 + item)
  action step = total' = total
  val safety = total == 312
  val broken = total == 123
}
