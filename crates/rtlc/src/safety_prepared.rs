use crate::Value;
use crate::ir::{Expression, SafetyProgram, TemporalFormula};
use crate::safety_expressions::register_branch_memberships;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct AddressHasher(u64);

impl Hasher for AddressHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3) ^ u64::from(*byte);
        }
    }

    fn write_usize(&mut self, value: usize) {
        // MurmurHash3 fmix64 by Austin Appleby (public domain), for internal addresses only.
        let mut hash = value as u64;
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        self.0 = hash ^ (hash >> 33);
    }
}

pub(crate) struct PreparedProgram<'a> {
    slots: HashMap<usize, usize, BuildHasherDefault<AddressHasher>>,
    values: Vec<&'a Value>,
}

impl<'a> PreparedProgram<'a> {
    pub fn new(program: &'a SafetyProgram) -> Self {
        let mut roots = Vec::new();
        roots.extend(
            program
                .definitions
                .iter()
                .flatten()
                .map(|definition| &definition.body),
        );
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
                TemporalFormula::Frame {
                    arguments,
                    captures,
                    formula,
                    ..
                } => {
                    roots.extend(arguments.iter().chain(captures));
                    formulas.push(formula);
                }
            }
        }
        Self::from_roots(roots)
    }

    pub fn from_roots(mut roots: Vec<&'a Expression>) -> Self {
        let mut prepared = Self {
            slots: HashMap::default(),
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
