import Cache

open Lean

def main : IO Unit := do
  let input ← (← IO.getStdin).readToEnd
  match Json.parse input >>= fromJson? (α := Array Cache.ReuseProbe) with
  | .error message => throw (IO.userError message)
  | .ok probes => (← IO.getStdout).putStrLn (toJson (probes.map Cache.reuseExpansion)).compress
