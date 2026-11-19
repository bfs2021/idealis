import Cache

open Lean

structure Probe where
  state : Cache.Cell
  action : Cache.Action
  deriving FromJson

def main : IO Unit := do
  let input ← (← IO.getStdin).readToEnd
  let parsed := Json.parse input >>= fromJson? (α := Array Probe)
  match parsed with
  | .error message => throw (IO.userError message)
  | .ok probes =>
    let replies := probes.map fun probe => Cache.step probe.state probe.action
    (← IO.getStdout).putStrLn (toJson replies).compress
