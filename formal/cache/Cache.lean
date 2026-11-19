import Std
import Lean.Data.Json

set_option autoImplicit false
set_option warningAsError true
set_option linter.unusedSimpArgs false

namespace Cache

structure Cell where
  identity : String
  fence : Nat := 0
  phase : String := "idle"
  owner : String := ""
  expiresAt : Nat := 0
  storedIdentity : String := ""
  complete : Bool := false
  deriving Repr, BEq, Lean.FromJson, Lean.ToJson

structure Action where
  op : String
  identity : String
  owner : String := ""
  fence : Nat := 0
  now : Nat := 0
  duration : Nat := 1
  receiptIdentity : String := ""
  complete : Bool := false
  deriving Repr, BEq, Lean.FromJson, Lean.ToJson

structure Reply where
  state : Cell
  code : String
  deriving Repr, BEq, Lean.FromJson, Lean.ToJson

def clear (cell : Cell) : Cell :=
  { identity := cell.identity, fence := cell.fence }

def holds (cell : Cell) (action : Action) : Bool :=
  cell.phase == "leased" && action.identity == cell.identity &&
  action.owner == cell.owner && action.fence == cell.fence

def live (cell : Cell) (action : Action) : Bool :=
  holds cell action && action.now < cell.expiresAt

def step (cell : Cell) (action : Action) : Reply :=
  if action.identity != cell.identity then ⟨cell, "mismatch"⟩
  else if action.op == "probe" then
    if cell.phase == "ready" then ⟨cell, "hit"⟩
    else if cell.phase == "leased" && action.now < cell.expiresAt then ⟨cell, "busy"⟩
    else ⟨cell, "miss"⟩
  else if action.op == "acquire" then
    if cell.phase == "ready" then ⟨cell, "hit"⟩
    else if cell.phase == "leased" && action.now < cell.expiresAt then ⟨cell, "busy"⟩
    else if action.duration == 0 then ⟨cell, "invalid"⟩
    else ⟨{ identity := cell.identity, fence := cell.fence + 1,
            phase := "leased", owner := action.owner, expiresAt := action.now + action.duration }, "lease"⟩
  else if action.op == "renew" then
    if live cell action && action.duration > 0 then
      ⟨{ cell with expiresAt := max cell.expiresAt (action.now + action.duration) }, "renewed"⟩
    else ⟨cell, "stale"⟩
  else if action.op == "publish" then
    if !live cell action then ⟨cell, "stale"⟩
    else if action.receiptIdentity != cell.identity then ⟨cell, "mismatch"⟩
    else if !action.complete then ⟨clear cell, "uncacheable"⟩
    else ⟨{ cell with phase := "ready", owner := "", expiresAt := 0,
                      storedIdentity := action.receiptIdentity, complete := true }, "published"⟩
  else if action.op == "release" then
    if holds cell action then ⟨clear cell, "released"⟩ else ⟨cell, "stale"⟩
  else if action.op == "evict" then
    if cell.phase == "ready" then ⟨clear cell, "evicted"⟩ else ⟨cell, "miss"⟩
  else ⟨cell, "invalid"⟩

def Valid (cell : Cell) : Prop :=
  cell.phase = "ready" → cell.complete = true ∧ cell.storedIdentity = cell.identity

theorem initial_valid (identity : String) : Valid { identity := identity } := by
  simp [Valid]

theorem step_valid (cell : Cell) (action : Action) (valid : Valid cell) :
    Valid (step cell action).state := by
  unfold step
  repeat' split
  all_goals simp_all [Valid, clear]

theorem step_identity (cell : Cell) (action : Action) :
    (step cell action).state.identity = cell.identity := by
  unfold step
  repeat' split
  all_goals simp_all [clear]

theorem step_fence_monotone (cell : Cell) (action : Action) :
    cell.fence ≤ (step cell action).state.fence := by
  unfold step
  repeat' split
  all_goals simp_all [clear]

theorem expired_publish (cell : Cell) (action : Action)
    (op : action.op = "publish") (expired : cell.expiresAt ≤ action.now) :
    (step cell action).state = cell := by
  simp [step, op, live, Nat.not_lt.mpr expired]
  split <;> rfl

theorem stale_fence_publish (cell : Cell) (action : Action)
    (op : action.op = "publish") (stale : action.fence ≠ cell.fence) :
    (step cell action).state = cell := by
  simp [step, op, live, holds, stale]
  split <;> rfl

theorem wrong_receipt_not_published (cell : Cell) (action : Action)
    (op : action.op = "publish") (wrong : action.receiptIdentity ≠ cell.identity) :
    (step cell action).code ≠ "published" := by
  simp [step, op, wrong]
  repeat' split
  all_goals simp_all

theorem incomplete_not_published (cell : Cell) (action : Action)
    (op : action.op = "publish") (incomplete : action.complete = false) :
    (step cell action).code ≠ "published" := by
  simp [step, op, incomplete]
  repeat' split
  all_goals simp_all

theorem current_complete_publishes (cell : Cell) (action : Action)
    (op : action.op = "publish") (current : live cell action = true)
    (same : action.identity = cell.identity)
    (receipt : action.receiptIdentity = cell.identity) (complete : action.complete = true) :
    (step cell action).code = "published" := by
  simp [step, op, current, same, receipt, complete]

theorem renewed_fence (cell : Cell) (action : Action) (op : action.op = "renew") :
    (step cell action).state.fence = cell.fence := by
  simp [step, op]
  repeat' split
  all_goals simp_all

theorem foreign_owner_publish (cell : Cell) (action : Action)
    (op : action.op = "publish") (foreign : action.owner ≠ cell.owner) :
    (step cell action).state = cell := by
  simp [step, op, live, holds, foreign]
  split <;> rfl

theorem acquired_fence (cell : Cell) (action : Action) (op : action.op = "acquire")
    (granted : (step cell action).code = "lease") :
    (step cell action).state.fence = cell.fence + 1 := by
  unfold step at granted ⊢
  repeat' split
  all_goals simp_all

theorem cache_hit_reachable :
    let initial : Cell := { identity := "model" }
    let acquired := (step initial { op := "acquire", identity := "model", owner := "worker", duration := 3 }).state
    let published := (step acquired { op := "publish", identity := "model", owner := "worker", fence := 1, now := 1, receiptIdentity := "model", complete := true }).state
    (step published { op := "probe", identity := "model", now := 10 }).code = "hit" := by
  decide

def run (cell : Cell) (actions : List Action) : Cell :=
  actions.foldl (fun state action => (step state action).state) cell

theorem trace_valid (cell : Cell) (actions : List Action) (valid : Valid cell) :
    Valid (run cell actions) := by
  induction actions generalizing cell with
  | nil => exact valid
  | cons action actions induction =>
    exact induction _ (step_valid cell action valid)

theorem trace_fence_monotone (cell : Cell) (actions : List Action) :
    cell.fence ≤ (run cell actions).fence := by
  induction actions generalizing cell with
  | nil => exact Nat.le_refl _
  | cons action actions induction =>
    exact Nat.le_trans (step_fence_monotone cell action) (induction _)

inductive Reachable {State : Type} (initial : State → Prop) (next : State → State → Prop) : State → Prop
  | root {state : State} : initial state → Reachable initial next state
  | edge {source target : State} : Reachable initial next source → next source target →
      Reachable initial next target

theorem closed_graph_covers {State : Type} (initial : State → Prop)
    (next : State → State → Prop) (cached : State → Prop)
    (roots : ∀ state, initial state → cached state)
    (closed : ∀ source target, cached source → next source target → cached target) :
    ∀ state, Reachable initial next state → cached state := by
  intro state reachable
  induction reachable with
  | root initialState => exact roots _ initialState
  | edge _ edge covered => exact closed _ _ covered edge

theorem cached_graph_safe {State : Type} (initial : State → Prop)
    (next : State → State → Prop) (cached property : State → Prop)
    (roots : ∀ state, initial state → cached state)
    (closed : ∀ source target, cached source → next source target → cached target)
    (checked : ∀ state, cached state → property state) :
    ∀ state, Reachable initial next state → property state := by
  intro state reachable
  exact checked state (closed_graph_covers initial next cached roots closed state reachable)

end Cache
