use crate::{
    AnonymousFrame, CompileError, Compiler, Context, E, Expression, Phase, Result, unsupported,
};
use crate::{
    analysis::{Node, NodeMut, analysis_children},
    typed,
};
use rtlc::ir::{self, Expression as I, Step, TemporalFormula as T};
use std::collections::HashSet;

fn branch(name: &str, steps: Vec<Step>) -> ir::Branch {
    ir::Branch {
        name: name.into(),
        steps,
    }
}
fn guard(expression: I) -> Step {
    Step::Guard { expression }
}
fn not(argument: I) -> I {
    I::Not {
        argument: Box::new(argument),
    }
}
fn atom(expression: I) -> T {
    T::Atom { expression }
}
fn negate(mut formula: T) -> T {
    if let T::Atom { expression } = &mut formula {
        atom(not(std::mem::replace(expression, I::Old)))
    } else {
        T::Not {
            formula: Box::new(formula),
        }
    }
}
fn combine(and: bool, mut left: T, mut right: T) -> T {
    match (&mut left, &mut right) {
        (T::Atom { expression: left }, T::Atom { expression: right }) => {
            let left = std::mem::replace(left, I::Old);
            let right = std::mem::replace(right, I::Old);
            atom(if and {
                I::And {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            } else {
                I::Or {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            })
        }
        _ => {
            if and {
                T::And {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            } else {
                T::Or {
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
        }
    }
}
fn frame_actuals(frame: &AnonymousFrame) -> (Vec<I>, Vec<I>) {
    (frame.arguments.clone(), frame.captures.clone())
}
fn frame_step(frame: &AnonymousFrame, branches: Vec<ir::Branch>) -> Result<Step> {
    let (arguments, captures) = frame_actuals(frame);
    let mut step = Step::Frame {
        parameter_count: arguments.len(),
        capture_count: captures.len(),
        local_scope: 0,
        arguments,
        captures,
        branches,
    };
    typed::compact_anonymous(NodeMut::Step(&mut step));
    Ok(step)
}
fn frame_expression(frame: &AnonymousFrame, body: I) -> Result<I> {
    let (arguments, captures) = frame_actuals(frame);
    let mut expression = I::Frame {
        parameter_count: arguments.len(),
        capture_count: captures.len(),
        local_scope: typed::local_scope(Node::Expression(&body)),
        arguments,
        captures,
        body: Box::new(body),
    };
    typed::compact_anonymous(NodeMut::Expression(&mut expression));
    Ok(expression)
}
fn frame_formula(frame: &AnonymousFrame, formula: T) -> Result<T> {
    let (arguments, captures) = frame_actuals(frame);
    let mut formula = T::Frame {
        parameter_count: arguments.len(),
        capture_count: captures.len(),
        local_scope: 0,
        arguments,
        captures,
        formula: Box::new(formula),
    };
    typed::compact_anonymous(NodeMut::Temporal(&mut formula));
    Ok(formula)
}
fn unchanged(value: I) -> Result<Vec<Step>> {
    let children = analysis_children(Node::Expression(&value));
    match &value {
        I::Tuple { .. } => children.into_iter().try_fold(vec![], |mut steps, edge| {
            if let Node::Expression(value) = edge.node {
                steps.extend(unchanged(value.clone())?);
            }
            Ok(steps)
        }),
        I::Read { slot, next: false } => Ok(vec![Step::Assign { slot: *slot, value }]),
        _ => unsupported("generative UNCHANGED requires state variables"),
    }
}
impl Compiler<'_> {
    fn target_reads_ir(&mut self, value: &Expression, context: &Context) -> Result<Vec<I>> {
        let target = Self::target_ir(&self.resolve_target(value, context)?)?;
        self.variable_reads_ir(target, "generative UNCHANGED requires state variables")
    }
    pub(super) fn action_ir(
        &mut self,
        value: &Expression,
        context: &Context,
        name: &str,
    ) -> Result<Vec<ir::Branch>> {
        let child = context.nested()?;
        match &value.node {
            E::BoxAction { action, subscript } | E::AngleAction { action, subscript }
                if context.phase == Phase::Next =>
            {
                let target = Self::target_ir(&self.resolve_target(subscript, context)?)?;
                let mut branches = self.action_ir(action, &child, name)?;
                if matches!(value.node, E::BoxAction { .. }) {
                    let mut steps = vec![];
                    for read in self.variable_reads_ir(
                        target,
                        "generative UNCHANGED requires state variables",
                    )? {
                        steps.extend(unchanged(read)?);
                    }
                    branches.push(branch(name, steps));
                } else {
                    let next = self.prime_ir(target.clone())?;
                    let expression = I::Ne {
                        left: Box::new(target),
                        right: Box::new(next),
                    };
                    for branch in &mut branches {
                        branch.steps.push(guard(expression.clone()));
                    }
                }
                typed::limited(branches)
            }
            E::Identifier { name } if self.has(context, name) => {
                self.action_call_ir(name, &[], context)
            }
            E::Apply { operator, args } if self.has(context, operator) => {
                self.action_call_ir(operator, args, context)
            }
            E::Let { definitions, body } => {
                let expanded = self.let_context(definitions, context)?;
                self.action_ir(body, &expanded, name)
            }
            E::Label { body, .. } => self.action_ir(body, &child, name),
            E::Exists { bounds, body } => {
                let (bindings, bound) = self.bind_ir(bounds, &child)?;
                let prefix: Vec<_> = bindings
                    .into_iter()
                    .map(|binding| Step::Bind {
                        slot: binding.slot,
                        tuple_size: binding.tuple_size,
                        domain: binding.domain,
                    })
                    .collect();
                let mut branches = self.action_ir(body, &bound, name)?;
                for branch in &mut branches {
                    branch.steps.splice(0..0, prefix.clone());
                }
                typed::limited(branches)
            }
            E::Or { operands, .. } => {
                let mut result = vec![];
                for operand in operands {
                    result.extend(self.action_ir(operand, &child, name)?);
                    result = typed::limited(result)?;
                }
                Ok(result)
            }
            E::And { operands, .. } => {
                let mut result = vec![branch(name, vec![])];
                for operand in operands {
                    let mut next = vec![];
                    for branch in &result {
                        let bound = context.bound
                            + branch
                                .steps
                                .iter()
                                .filter(|step| matches!(step, Step::Bind { .. }))
                                .count();
                        for mut addition in self.action_ir(
                            operand,
                            &Context {
                                bound,
                                ..child.clone()
                            },
                            name,
                        )? {
                            addition.steps.splice(0..0, branch.steps.clone());
                            next.push(addition);
                            if next.len() > 4096 {
                                return unsupported("action expansion exceeds 4096 branches");
                            }
                        }
                    }
                    result = typed::limited(next)?;
                }
                Ok(result)
            }
            E::If {
                condition,
                consequent,
                alternative,
            } => {
                let condition = self.expression_ir(condition, &child)?;
                let mut result = vec![];
                for (body, condition) in [
                    (consequent, condition.clone()),
                    (alternative, not(condition)),
                ] {
                    for mut branch in self.action_ir(body, &child, name)? {
                        branch.steps.insert(0, guard(condition.clone()));
                        result.push(branch);
                    }
                }
                typed::limited(result)
            }
            E::Case { arms, other } if !arms.is_empty() => {
                let mut result = vec![];
                let mut previous: Vec<I> = vec![];
                for arm in arms {
                    let condition = self.expression_ir(&arm.condition, &child)?;
                    let mut guards: Vec<_> = previous.iter().cloned().map(not).map(guard).collect();
                    guards.push(guard(condition.clone()));
                    for mut branch in self.action_ir(&arm.body, &child, name)? {
                        branch.steps.splice(0..0, guards.clone());
                        result.push(branch);
                    }
                    previous.push(condition);
                    result = typed::limited(result)?;
                }
                let guards: Vec<_> = previous.into_iter().map(not).map(guard).collect();
                if let Some(other) = other {
                    for mut branch in self.action_ir(other, &child, name)? {
                        branch.steps.splice(0..0, guards.clone());
                        result.push(branch);
                    }
                } else {
                    let mut steps = guards;
                    steps.push(guard(I::UnmatchedCase {
                        location: format!(
                            "line {}, column {}",
                            value.location.line, value.location.column
                        ),
                    }));
                    result.push(branch(name, steps));
                }
                typed::limited(result)
            }
            E::Unchanged { expression } if context.phase == Phase::Next => {
                let mut steps = vec![];
                for read in self.target_reads_ir(expression, &child)? {
                    steps.extend(unchanged(read)?);
                }
                Ok(vec![branch(name, steps)])
            }
            E::Apply { operator, args } if args.len() == 2 => {
                let equality = operator == "=" || operator == ":=" && self.extends("Apalache");
                if let Some(slot) = self.target(&args[0], context) {
                    if equality {
                        return Ok(vec![branch(
                            name,
                            vec![Step::Assign {
                                slot,
                                value: self.expression_ir(&args[1], &child)?,
                            }],
                        )]);
                    }
                    if operator == "\\in" {
                        return Ok(vec![branch(
                            name,
                            vec![Step::Choose {
                                slot,
                                domain: self.domain_ir(&args[1], &child, true)?,
                            }],
                        )]);
                    }
                }
                if equality && let Some(slot) = self.target(&args[1], context) {
                    return Ok(vec![branch(
                        name,
                        vec![Step::Assign {
                            slot,
                            value: self.expression_ir(&args[0], &child)?,
                        }],
                    )]);
                }
                Ok(vec![branch(
                    name,
                    vec![guard(self.expression_ir(value, context)?)],
                )])
            }
            _ => Ok(vec![branch(
                name,
                vec![guard(self.expression_ir(value, context)?)],
            )]),
        }
    }
    fn action_call_ir(
        &mut self,
        name: &str,
        args: &[Expression],
        context: &Context,
    ) -> Result<Vec<ir::Branch>> {
        if !self.generative_call(name, context, &mut HashSet::new())? {
            return Ok(vec![branch(
                name,
                vec![guard(self.shared_ir(name, args, context)?)],
            )]);
        }
        let (body, expanded) = self.call(name, args, context)?;
        let branches = self.action_ir(&body, &expanded, name)?;
        if let Some(frame) = &expanded.frame {
            typed::limited(vec![branch(name, vec![frame_step(frame, branches)?])])
        } else {
            Ok(branches)
        }
    }
    pub(super) fn temporal_ir(&mut self, value: &Expression, context: &Context) -> Result<T> {
        let child = context.nested()?;
        match &value.node {
            E::Forall { bounds, body } | E::Exists { bounds, body } => {
                let (bindings, bound) = self.bind_ir(bounds, &child)?;
                let formula = self.temporal_ir(body, &bound)?;
                if matches!(formula, T::Atom { .. }) {
                    return Ok(atom(self.expression_ir(value, &child)?));
                }
                self.lower_pending()?;
                let bounds = bindings;
                if bounds.iter().any(|bound| {
                    typed::level_summary(Node::Domain(&bound.domain), &self.level_cache.borrow())
                        .base
                        > 0
                }) {
                    return unsupported("temporal quantification requires constant finite domains");
                }
                Ok(if matches!(value.node, E::Forall { .. }) {
                    T::Forall {
                        bounds,
                        formula: Box::new(formula),
                    }
                } else {
                    T::Exists {
                        bounds,
                        formula: Box::new(formula),
                    }
                })
            }
            E::Identifier { name } if self.has(context, name) => {
                self.temporal_call_ir(name, &[], context)
            }
            E::Apply { operator, args } if self.has(context, operator) => {
                self.temporal_call_ir(operator, args, context)
            }
            E::Let { definitions, body } => {
                let expanded = self.let_context(definitions, context)?;
                self.temporal_ir(body, &expanded)
            }
            E::Always { expression } | E::Eventually { expression } => {
                let formula = self.temporal_ir(expression, &child)?;
                self.lower_pending()?;
                let always = matches!(value.node, E::Always { .. });
                if always && let Some(mut body) = typed::atom_expression(&formula) {
                    for frame in context.temporal_frames.iter().rev() {
                        body = frame_expression(frame, body)?;
                    }
                    if typed::level_summary(Node::Expression(&body), &self.level_cache.borrow())
                        .base
                        > 1
                        && !self.boxed_action(expression, &child)?
                    {
                        return unsupported("[] action must have the form [A]_v");
                    }
                }
                Ok(if always {
                    T::Always {
                        formula: Box::new(formula),
                    }
                } else {
                    T::Eventually {
                        formula: Box::new(formula),
                    }
                })
            }
            E::And { operands, .. } | E::Or { operands, .. } => {
                let and = matches!(value.node, E::And { .. });
                let mut result = atom(I::Literal {
                    value: ir::Scalar::Bool(and),
                });
                for operand in operands {
                    result = combine(and, result, self.temporal_ir(operand, &child)?);
                }
                Ok(result)
            }
            E::Apply { operator, args }
                if args.len() == 1 && ["~", "\\lnot"].contains(&operator.as_str()) =>
            {
                Ok(negate(self.temporal_ir(&args[0], &child)?))
            }
            E::Apply { operator, args }
                if args.len() == 2 && ["=>", "~>"].contains(&operator.as_str()) =>
            {
                let right = self.temporal_ir(&args[1], &child)?;
                let result = combine(
                    false,
                    negate(self.temporal_ir(&args[0], &child)?),
                    if operator == "~>" {
                        T::Eventually {
                            formula: Box::new(right),
                        }
                    } else {
                        right
                    },
                );
                Ok(if operator == "~>" {
                    T::Always {
                        formula: Box::new(result),
                    }
                } else {
                    result
                })
            }
            E::WeakFairness { action, subscript } | E::StrongFairness { action, subscript } => {
                let subscript = self.expression_ir(subscript, &child)?;
                let enabled = atom(I::Enabled {
                    branches: self.action_ir(action, &child, "fairness")?,
                    subscript: Some(Box::new(subscript.clone())),
                });
                let disabled = T::Not {
                    formula: Box::new(enabled),
                };
                let primed = self.prime_ir(subscript.clone())?;
                let taken = atom(I::And {
                    left: Box::new(self.expression_ir(action, &child)?),
                    right: Box::new(I::Ne {
                        left: Box::new(subscript),
                        right: Box::new(primed),
                    }),
                });
                Ok(if matches!(value.node, E::WeakFairness { .. }) {
                    T::Always {
                        formula: Box::new(T::Eventually {
                            formula: Box::new(T::Or {
                                left: Box::new(disabled),
                                right: Box::new(taken),
                            }),
                        }),
                    }
                } else {
                    T::Or {
                        left: Box::new(T::Eventually {
                            formula: Box::new(T::Always {
                                formula: Box::new(disabled),
                            }),
                        }),
                        right: Box::new(T::Always {
                            formula: Box::new(T::Eventually {
                                formula: Box::new(taken),
                            }),
                        }),
                    }
                })
            }
            _ => Ok(atom(self.expression_ir(value, context)?)),
        }
    }
    fn temporal_call_ir(
        &mut self,
        name: &str,
        args: &[Expression],
        context: &Context,
    ) -> Result<T> {
        let (body, expanded) = self.call(name, args, context)?;
        let formula = self.temporal_ir(&body, &expanded)?;
        if let Some(frame) = &expanded.frame {
            frame_formula(frame, formula)
        } else {
            Ok(formula)
        }
    }
}

impl Compiler<'_> {
    fn parts_ir(
        &mut self,
        value: &Expression,
        context: &Context,
        initial: &mut Option<Vec<ir::Branch>>,
        next: &mut Option<Vec<ir::Branch>>,
        fairness: &mut Vec<T>,
    ) -> Result<()> {
        match &value.node {
            E::Identifier { name } if self.has(context, name) => {
                let (body, expanded) = self.call(name, &[], context)?;
                self.parts_ir(&body, &expanded, initial, next, fairness)
            }
            E::And { .. } => {
                for edge in analysis_children(Node::Source(value))
                    .into_iter()
                    .filter(|edge| edge.descriptor.action == "inherit")
                {
                    if let Node::Source(operand) = edge.node {
                        self.parts_ir(operand, &context.nested()?, initial, next, fairness)?;
                    }
                }
                Ok(())
            }
            E::Always { expression } if matches!(expression.node, E::BoxAction { .. }) => {
                if next.is_some() {
                    return unsupported("multiple NEXT conjuncts");
                }
                let children = analysis_children(Node::Source(expression));
                let action = children
                    .iter()
                    .find_map(|edge| {
                        if edge.descriptor.role == "action-body" {
                            if let Node::Source(value) = edge.node {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| CompileError {
                        message: "SPECIFICATION action".into(),
                    })?;
                let subscript = children
                    .iter()
                    .find_map(|edge| {
                        if edge.descriptor.role == "subscript" {
                            if let Node::Source(value) = edge.node {
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| CompileError {
                        message: "SPECIFICATION subscript".into(),
                    })?;
                let subscript = self.expression_ir(subscript, context)?;
                let mut reads = HashSet::new();
                for value in self.variable_reads_ir(
                    subscript,
                    "SPECIFICATION subscript must list state variables",
                )? {
                    read_slots(&value, &mut reads)?;
                }
                if reads.len() != self.slots.len() {
                    return unsupported("SPECIFICATION subscript must cover every state variable");
                }
                *next = Some(self.action_ir(action, context, "Next")?);
                Ok(())
            }
            E::WeakFairness { .. } | E::StrongFairness { .. } => {
                fairness.push(self.temporal_ir(value, context)?);
                Ok(())
            }
            _ => {
                let additions = self.action_ir(
                    value,
                    &Context {
                        phase: Phase::Initial,
                        ..context.clone()
                    },
                    "Init",
                )?;
                *initial = Some(if let Some(previous) = initial.take() {
                    let mut result = vec![];
                    for branch in &previous {
                        for addition in &additions {
                            let mut joined = addition.clone();
                            joined.steps.splice(0..0, branch.steps.clone());
                            result.push(joined);
                            if result.len() > 4096 {
                                return unsupported("action expansion exceeds 4096 branches");
                            }
                        }
                    }
                    typed::limited(result)?
                } else {
                    additions
                });
                Ok(())
            }
        }
    }
    pub(super) fn program_ir(&mut self) -> Result<ir::SafetyProgram> {
        let mut initial = None;
        let mut next = None;
        let mut fairness = vec![];
        if let Some(name) = self.config.specification.clone() {
            let (body, context) = self.operator(&name, Phase::Next)?;
            self.parts_ir(&body, &context, &mut initial, &mut next, &mut fairness)?;
        } else {
            for (phase, name) in [
                (Phase::Initial, self.config.init.clone()),
                (Phase::Next, self.config.next.clone()),
            ] {
                let name = name.ok_or_else(|| CompileError {
                    message: "INIT / NEXT".into(),
                })?;
                let (body, context) = self.operator(&name, phase)?;
                let branches = self.action_ir(&body, &context, &name)?;
                if phase == Phase::Initial {
                    initial = Some(branches);
                } else {
                    next = Some(branches);
                }
            }
        }
        let initial = initial.ok_or_else(|| CompileError {
            message: "initial predicate required".into(),
        })?;
        let next = next.ok_or_else(|| CompileError {
            message: "NEXT required".into(),
        })?;
        self.lower_pending()?;
        if !next.iter().any(|branch| {
            analysis_children(Node::Branch(branch))
                .into_iter()
                .any(|edge| {
                    if let Node::Step(step) = edge.node {
                        if let Step::Guard { expression } = step {
                            typed::level_summary(
                                Node::Expression(expression),
                                &self.level_cache.borrow(),
                            )
                            .base
                                > 0
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                })
        }) {
            return unsupported("NEXT cannot be constant");
        }
        let mut program = ir::SafetyProgram {
            format: "modelt.safety-program".into(),
            version: 2,
            name: self.module.name.clone(),
            variables: self
                .module
                .variables
                .iter()
                .map(|value| value.name.clone())
                .collect(),
            initial,
            next,
            invariants: vec![],
            assumptions: vec![],
            check_deadlock: self.config.check_deadlock,
            symmetry: None,
            initial_properties: vec![],
            action_properties: vec![],
            properties: vec![],
            fairness: vec![],
            constraints: vec![],
            action_constraints: vec![],
            definitions: Some(vec![]),
        };
        for name in self.config.invariants.clone() {
            program.invariants.push(ir::Predicate {
                expression: self.shared_ir(&name, &[], &self.root(Phase::State))?,
                name,
            });
        }
        for name in self.config.properties.clone() {
            let (body, context) = self.operator(&name, Phase::Next)?;
            let formula = self.temporal_ir(&body, &context)?;
            self.lower_pending()?;
            let initial_expression = typed::atom_expression(&formula);
            let always_expression = typed::always_expression(&formula);
            typed::validate_temporal(
                &formula,
                &[],
                always_expression.is_some(),
                &[],
                &self.level_cache.borrow(),
            )?;
            if let Some(expression) = initial_expression {
                if typed::level_summary(Node::Expression(&expression), &self.level_cache.borrow())
                    .base
                    > 1
                {
                    return unsupported("initial PROPERTY must be a state predicate");
                }
                program
                    .initial_properties
                    .push(ir::Predicate { name, expression });
            } else if let Some(expression) = always_expression {
                let target = if typed::level_summary(
                    Node::Expression(&expression),
                    &self.level_cache.borrow(),
                )
                .base
                    > 1
                {
                    &mut program.action_properties
                } else {
                    &mut program.invariants
                };
                target.push(ir::Predicate { name, expression });
            } else {
                program
                    .properties
                    .push(ir::TemporalProperty { name, formula });
            }
        }
        for (index, assumption) in self.module.assumptions.iter().enumerate() {
            program.assumptions.push(ir::Predicate {
                name: assumption
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("ASSUME {}", index + 1)),
                expression: self
                    .expression_ir(&assumption.expression, &self.root(Phase::Constant))?,
            });
        }
        for (names, phase) in [
            (self.config.constraints.clone(), Phase::State),
            (self.config.action_constraints.clone(), Phase::Next),
        ] {
            for name in names {
                let predicate = ir::Predicate {
                    expression: self.shared_ir(&name, &[], &self.root(phase))?,
                    name,
                };
                if phase == Phase::State {
                    program.constraints.push(predicate);
                } else {
                    program.action_constraints.push(predicate);
                }
            }
        }
        if !program.properties.is_empty() {
            program.fairness = fairness;
        }
        self.finish_shared(program)
    }
}
fn read_slots(value: &I, slots: &mut HashSet<usize>) -> Result<()> {
    let children = analysis_children(Node::Expression(value));
    match value {
        I::Tuple { .. } => {
            for edge in children {
                if let Node::Expression(value) = edge.node {
                    read_slots(value, slots)?;
                }
            }
            Ok(())
        }
        I::Read { slot, next: false } => {
            slots.insert(*slot);
            Ok(())
        }
        _ => unsupported("SPECIFICATION subscript must list state variables"),
    }
}
