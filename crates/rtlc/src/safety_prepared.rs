use crate::Value;
use crate::ir::{Expression, SafetyProgram, TemporalFormula};
use crate::safety_expressions::register_branch_memberships;
use std::collections::HashMap;

pub(crate) struct PreparedProgram<'a> {
    slots: HashMap<usize, usize>,
    values: Vec<&'a Value>,
}

impl<'a> PreparedProgram<'a> {
    pub fn new(program: &'a SafetyProgram) -> Self {
        let mut roots = Vec::new();
        register_branch_memberships(&program.initial, &mut |expression| roots.push(expression));
        register_branch_memberships(&program.next, &mut |expression| roots.push(expression));
        roots.extend(
            program
                .invariants
                .iter()
                .chain(&program.assumptions)
                .chain(&program.initial_properties)
                .chain(&program.action_properties)
                .chain(&program.constraints)
                .chain(&program.action_constraints)
                .map(|predicate| &predicate.expression),
        );
        let mut formulas = program
            .properties
            .iter()
            .map(|property| &property.formula)
            .chain(&program.fairness)
            .collect::<Vec<_>>();
        while let Some(formula) = formulas.pop() {
            match formula {
                TemporalFormula::Atom { expression } => roots.push(expression),
                TemporalFormula::Not { formula }
                | TemporalFormula::Always { formula }
                | TemporalFormula::Eventually { formula } => formulas.push(formula),
                TemporalFormula::And { left, right } | TemporalFormula::Or { left, right } => {
                    formulas.extend([left.as_ref(), right.as_ref()])
                }
                TemporalFormula::Forall { bounds, formula }
                | TemporalFormula::Exists { bounds, formula } => {
                    for bound in bounds {
                        roots.extend(bound.domain.children());
                    }
                    formulas.push(formula);
                }
            }
        }
        Self::from_roots(roots)
    }

    pub fn from_roots(mut roots: Vec<&'a Expression>) -> Self {
        let mut prepared = Self {
            slots: HashMap::new(),
            values: Vec::new(),
        };
        while let Some(expression) = roots.pop() {
            if let Expression::Data { value } = expression {
                let address = std::ptr::from_ref(expression) as usize;
                if let std::collections::hash_map::Entry::Vacant(entry) =
                    prepared.slots.entry(address)
                {
                    entry.insert(prepared.values.len());
                    prepared.values.push(value);
                }
            }
            if let Expression::Enabled {
                branches,
                subscript,
            } = expression
            {
                register_branch_memberships(branches, &mut |expression| roots.push(expression));
                roots.extend(subscript.iter().map(Box::as_ref));
            }
            roots.extend(expression.children());
        }
        prepared
    }

    pub fn slot(&self, expression: &Expression) -> Option<usize> {
        self.slots
            .get(&(std::ptr::from_ref(expression) as usize))
            .copied()
    }

    pub fn value(&self, slot: usize) -> &'a Value {
        self.values[slot]
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
