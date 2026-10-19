///|
/// What a tally is, on this side of the boundary. The component that holds
/// a handle never sees this; it only ever sees a number.
priv struct State {
  name : String
  mut total : UInt
}

///|
/// From each resource's rep to its state.
let states : Map[Int, State] = Map([])

///|
let next_rep : Ref[Int] = { val: 0 }

///|
fn log(line : String) -> Unit {
  let bytes = @utf8.encode(line + "\n")
  let stream = @stderr.get_stderr()
  stream.blocking_write_and_flush(bytes.to_fixedarray()) |> ignore
  stream.drop()
}

///|
pub fn Tally::tally(name : String) -> Tally {
  next_rep.val += 1
  let rep = next_rep.val
  states[rep] = { name, total: 0 }
  log("tally: created \{name}")
  Tally::new(rep)
}

///|
pub fn Tally::add(self : Tally, n : UInt) -> UInt {
  let Tally(rep) = self
  let state = states[rep]
  state.total += n
  state.total
}

///|
pub fn Tally::dtor(self : Tally) -> Unit {
  let Tally(rep) = self
  let state = states[rep]
  log("tally: dropped \{state.name} at \{state.total}")
  states.remove(rep)
}
