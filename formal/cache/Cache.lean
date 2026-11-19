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

structure ReuseProbe where
  enabled : Bool := false
  sameBuild : Bool
  sameNext : Bool
  sameState : Bool
  complete : Bool
  deriving Lean.FromJson, Lean.ToJson

def reuseExpansion (probe : ReuseProbe) : Bool :=
  probe.enabled && probe.sameBuild && probe.sameState && probe.sameNext && probe.complete

theorem disabled_expansion_not_reused (probe : ReuseProbe) (disabled : probe.enabled = false) :
    reuseExpansion probe = false := by simp [reuseExpansion, disabled]

theorem changed_next_not_reused (probe : ReuseProbe) (changed : probe.sameNext = false) :
    reuseExpansion probe = false := by simp [reuseExpansion, changed]

theorem partial_expansion_not_reused (probe : ReuseProbe) (unfinished : probe.complete = false) :
    reuseExpansion probe = false := by simp [reuseExpansion, unfinished]

theorem changed_build_not_reused (probe : ReuseProbe) (changed : probe.sameBuild = false) :
    reuseExpansion probe = false := by simp [reuseExpansion, changed]

theorem different_state_not_reused (probe : ReuseProbe) (changed : probe.sameState = false) :
    reuseExpansion probe = false := by simp [reuseExpansion, changed]

theorem complete_expansion_reused :
    reuseExpansion ⟨true, true, true, true, true⟩ = true := by decide

def memoNext {State : Type} (current stored : State → State → Prop)
    (enabled : State → Bool) (source target : State) : Prop :=
  if enabled source then stored source target else current source target

theorem memo_next_exact {State : Type} (current stored : State → State → Prop)
    (enabled : State → Bool)
    (sound : ∀ source, enabled source = true → ∀ target, stored source target ↔ current source target) :
    ∀ source target, memoNext current stored enabled source target ↔ current source target := by
  intro source target
  unfold memoNext
  split
  next cached => exact sound source cached target
  next => exact Iff.rfl

theorem reachable_congr {State : Type} (initial : State → Prop)
    (left right : State → State → Prop)
    (same : ∀ source target, left source target ↔ right source target) :
    ∀ state, Reachable initial left state ↔ Reachable initial right state := by
  intro state
  constructor
  · intro reachable
    induction reachable with
    | root root => exact .root root
    | edge _ edge reached => exact .edge reached ((same _ _).mp edge)
  · intro reachable
    induction reachable with
    | root root => exact .root root
    | edge _ edge reached => exact .edge reached ((same _ _).mpr edge)

theorem memo_reachable_exact {State : Type} (initial : State → Prop)
    (current stored : State → State → Prop) (enabled : State → Bool)
    (sound : ∀ source, enabled source = true → ∀ target, stored source target ↔ current source target) :
    ∀ state, Reachable initial (memoNext current stored enabled) state ↔ Reachable initial current state := by
  exact reachable_congr initial _ current (memo_next_exact current stored enabled sound)

theorem rechecked_memo_safe {State : Type} (initial property : State → Prop)
    (current stored : State → State → Prop) (enabled : State → Bool)
    (sound : ∀ source, enabled source = true → ∀ target, stored source target ↔ current source target)
    (checked : ∀ state, Reachable initial (memoNext current stored enabled) state → property state) :
    ∀ state, Reachable initial current state → property state := by
  intro state reachable
  exact checked state ((memo_reachable_exact initial current stored enabled sound state).mpr reachable)

end Cache
