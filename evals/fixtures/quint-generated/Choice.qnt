module Choice {
  var value: int
  action init = value' = 0
  action step = {
    nondet next = Set(0, 1, 2).oneOf()
    value' = next
  }
  val safety = value >= 0 and value <= 2
  val broken = value < 2
}
