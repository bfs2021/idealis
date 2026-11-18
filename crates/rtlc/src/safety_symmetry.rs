use crate::ir::{Domain, Expression, SafetyProgram, Scalar, Step};
use crate::safety_values::Arena;
use crate::{CoreError, Value};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

struct Representative {
    values: Box<[Scalar]>,
    index: usize,
}

pub(crate) struct Symmetry {
    group: Vec<Scalar>,
    keys: HashMap<u64, Vec<Representative>>,
}

impl Symmetry {
    pub fn prepare(program: &SafetyProgram, arena: &mut Arena) -> Result<Self, CoreError> {
        let definition = program.symmetry.as_ref().unwrap();
        let mut names = definition.model_values.clone();
        names.sort();
        if !(2..=64).contains(&names.len())
            || names.iter().collect::<HashSet<_>>().len() != names.len()
            || names.iter().any(|name| {
                name.as_bytes().get(1) == Some(&b'_')
                    || Value::ModelValue(name.clone()).normalize().is_err()
            })
        {
            return Err(CoreError::protocol(
                "Unsupported symmetry: expected 2 to 64 distinct untyped model values",
            ));
        }
        if !program.properties.is_empty()
            || !program.fairness.is_empty()
            || !program.constraints.is_empty()
            || !program.action_constraints.is_empty()
            || !program.initial_properties.is_empty()
            || !program.action_properties.is_empty()
        {
            return Err(CoreError::protocol(
                "Unsupported symmetry: only unconstrained safety invariants and deadlock",
            ));
        }
        let mut expressions = Vec::new();
        for branch in program.initial.iter().chain(&program.next) {
            for step in &branch.steps {
                match step {
                    Step::Guard { expression }
                    | Step::Assign {
                        value: expression, ..
                    } => expressions.push(expression),
                    Step::Bind { domain, .. } | Step::Choose { domain, .. } => {
                        domain_roots(domain, &names, &mut expressions)
                    }
                }
            }
        }
        expressions.extend(
            program
                .invariants
                .iter()
                .chain(&program.assumptions)
                .map(|predicate| &predicate.expression),
        );
        while let Some(expression) = expressions.pop() {
            if node_set(expression, &names) {
                continue;
            }
            match expression {
                Expression::Data { value } => {
                    let mut values = vec![value];
                    while let Some(value) = values.pop() {
                        match value {
                            Value::ModelValue(_) => {
                                return Err(CoreError::protocol(
                                    "Unsupported symmetry: individual model value reference",
                                ));
                            }
                            Value::Set(items) | Value::List(items) => values.extend(items),
                            Value::Record(fields) => {
                                values.extend(fields.iter().map(|(_, value)| value))
                            }
                            Value::Map(entries) => {
                                values.extend(entries.iter().flat_map(|(key, value)| [key, value]))
                            }
                            _ => {}
                        }
                    }
                }
                Expression::Literal {
                    value: Scalar::Ref(_),
                }
                | Expression::Choose { .. }
                | Expression::Fold { .. }
                | Expression::SelectSeq { .. }
                | Expression::SortSeq { .. }
                | Expression::Enabled { .. }
                | Expression::UnmatchedCase { .. }
                | Expression::Assert { .. }
                | Expression::VariantTag { .. }
                | Expression::Head { .. }
                | Expression::Tail { .. }
                | Expression::Subseq { .. }
                | Expression::SequenceSet { .. }
                | Expression::SequenceIn { .. }
                | Expression::Append { .. }
                | Expression::Concat { .. }
                | Expression::Len { .. } => {
                    return Err(CoreError::protocol(
                        "Unsupported symmetry: order-sensitive or unsupported expression",
                    ));
                }
                Expression::Forall { bounds, body }
                | Expression::Exists { bounds, body }
                | Expression::Function { bounds, body }
                | Expression::Filter { bounds, body }
                | Expression::MapSet { bounds, body } => {
                    for bound in bounds {
                        domain_roots(&bound.domain, &names, &mut expressions);
                    }
                    expressions.push(body);
                    continue;
                }
                Expression::In { value, domain } => {
                    expressions.push(value);
                    domain_roots(domain, &names, &mut expressions);
                    continue;
                }
                _ => {}
            }
            expressions.extend(expression.children());
        }
        let group = names
            .into_iter()
            .map(|name| arena.data(&Value::ModelValue(name)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            group,
            keys: HashMap::new(),
        })
    }

    pub fn key(
        &self,
        values: &[Option<Scalar>],
        arena: &mut Arena,
    ) -> Result<Vec<Scalar>, CoreError> {
        arena.consume(values.len())?;
        let mut key = Vec::with_capacity(values.len());
        let mut rows = vec![Vec::new(); self.group.len()];
        let mut columns = 0;
        for value in values {
            match value.ok_or_else(|| CoreError::protocol("Incomplete safety state"))? {
                value @ (Scalar::Bool(_) | Scalar::Int(_)) => key.push(value),
                value @ Scalar::Ref(_) => {
                    arena.consume(self.group.len())?;
                    let entries = arena.entries(value).map_err(|_| CoreError::protocol("Unsupported symmetry: state must contain scalar node functions or Int/Bool"))?;
                    if entries.len() != self.group.len()
                        || !entries
                            .iter()
                            .zip(&self.group)
                            .all(|((key, _), member)| key == member)
                        || !entries
                            .iter()
                            .all(|(_, value)| matches!(value, Scalar::Int(_) | Scalar::Bool(_)))
                        || !entries.iter().all(|(_, value)| {
                            std::mem::discriminant(value) == std::mem::discriminant(&entries[0].1)
                        })
                    {
                        return Err(CoreError::protocol(
                            "Unsupported symmetry: node function needs the complete domain and homogeneous Int/Bool payload",
                        ));
                    }
                    for (row, (_, value)) in rows.iter_mut().zip(entries) {
                        row.push(value);
                    }
                    columns += 1;
                    key.push(Scalar::Ref(0));
                }
            }
        }
        if columns != 0 {
            arena.consume(self.group.len() * self.group.len() * columns)?;
            rows.sort_unstable();
            key.extend(rows.into_iter().flatten());
        }
        Ok(key)
    }

    pub fn hash(key: &[Scalar]) -> u64 {
        let mut hash = DefaultHasher::new();
        key.hash(&mut hash);
        hash.finish()
    }

    pub fn find(&self, hash: u64, key: &[Scalar]) -> Option<usize> {
        self.keys
            .get(&hash)?
            .iter()
            .find_map(|stored| (stored.values.as_ref() == key).then_some(stored.index))
    }

    pub fn insert(&mut self, hash: u64, key: Vec<Scalar>, index: usize) {
        self.keys.entry(hash).or_default().push(Representative {
            values: key.into_boxed_slice(),
            index,
        });
    }
}

fn node_members<'a>(values: impl Iterator<Item = Option<&'a str>>, names: &[String]) -> bool {
    let Some(mut values) = values
        .map(|value| value.map(str::to_owned))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    values.sort();
    values == names
}

fn node_set(expression: &Expression, names: &[String]) -> bool {
    match expression {
        Expression::Set { values } => node_members(
            values.iter().map(|value| match value {
                Expression::Data {
                    value: Value::ModelValue(name),
                } => Some(name.as_str()),
                _ => None,
            }),
            names,
        ),
        Expression::Data {
            value: Value::Set(values),
        } => node_members(
            values.iter().map(|value| match value {
                Value::ModelValue(name) => Some(name.as_str()),
                _ => None,
            }),
            names,
        ),
        _ => false,
    }
}

fn domain_roots<'a>(domain: &'a Domain, names: &[String], expressions: &mut Vec<&'a Expression>) {
    if let Domain::Set { values } = domain
        && node_members(
            values.iter().map(|value| match value {
                Expression::Data {
                    value: Value::ModelValue(name),
                } => Some(name.as_str()),
                _ => None,
            }),
            names,
        )
    {
        return;
    }
    expressions.extend(domain.children());
}

#[cfg(test)]
mod tests {
    use super::Symmetry;
    use crate::Value;
    use crate::ir::Scalar;
    use crate::safety_values::Arena;
    use std::collections::HashMap;

    fn symmetry(arena: &mut Arena) -> Symmetry {
        Symmetry {
            group: ["a", "b"]
                .into_iter()
                .map(|name| arena.data(&Value::ModelValue(name.into())).unwrap())
                .collect(),
            keys: HashMap::new(),
        }
    }
    fn function(arena: &mut Arena, symmetry: &Symmetry, values: [Scalar; 2]) -> Scalar {
        arena
            .function(symmetry.group.iter().copied().zip(values).collect())
            .unwrap()
    }

    #[test]
    fn joint_rows_preserve_correlations_global_values_and_variable_positions() {
        let mut arena = Arena::new(10000);
        let symmetry = symmetry(&mut arena);
        let forward = function(&mut arena, &symmetry, [Scalar::Int(0), Scalar::Int(1)]);
        let reverse = function(&mut arena, &symmetry, [Scalar::Int(1), Scalar::Int(0)]);
        let actual = symmetry
            .key(
                &[Some(forward), Some(forward), Some(Scalar::Bool(false))],
                &mut arena,
            )
            .unwrap();
        assert_eq!(
            actual,
            symmetry
                .key(
                    &[Some(reverse), Some(reverse), Some(Scalar::Bool(false))],
                    &mut arena
                )
                .unwrap()
        );
        assert_ne!(
            actual,
            symmetry
                .key(
                    &[Some(forward), Some(reverse), Some(Scalar::Bool(false))],
                    &mut arena
                )
                .unwrap()
        );
        assert_ne!(
            actual,
            symmetry
                .key(
                    &[Some(forward), Some(forward), Some(Scalar::Bool(true))],
                    &mut arena
                )
                .unwrap()
        );
        assert_ne!(
            actual,
            symmetry
                .key(
                    &[Some(Scalar::Bool(false)), Some(forward), Some(forward)],
                    &mut arena
                )
                .unwrap()
        );
    }

    #[test]
    fn colliding_canonical_hashes_use_the_complete_key() {
        let mut arena = Arena::new(100);
        let mut symmetry = symmetry(&mut arena);
        symmetry.insert(0, vec![Scalar::Int(1)], 7);
        symmetry.insert(0, vec![Scalar::Int(2)], 8);
        assert_eq!(symmetry.find(0, &[Scalar::Int(1)]), Some(7));
        assert_eq!(symmetry.find(0, &[Scalar::Int(2)]), Some(8));
        assert_eq!(symmetry.find(0, &[Scalar::Bool(true)]), None);
        assert_eq!(symmetry.find(0, &[Scalar::Int(1), Scalar::Int(2)]), None);
    }

    #[test]
    fn unsupported_shapes_and_incomplete_keys_are_errors() {
        let mut arena = Arena::new(10000);
        let symmetry = symmetry(&mut arena);
        let missing = arena
            .function(vec![(symmetry.group[0], Scalar::Int(0))])
            .unwrap();
        let mixed = function(&mut arena, &symmetry, [Scalar::Int(0), Scalar::Bool(false)]);
        let nested = function(
            &mut arena,
            &symmetry,
            [symmetry.group[0], symmetry.group[1]],
        );
        let name = arena.string("a".into()).unwrap();
        let strings = arena
            .function(vec![
                (name, Scalar::Int(0)),
                (symmetry.group[1], Scalar::Int(0)),
            ])
            .unwrap();
        for value in [
            Some(missing),
            Some(mixed),
            Some(nested),
            Some(name),
            Some(strings),
            Some(symmetry.group[0]),
            None,
        ] {
            assert_eq!(
                symmetry.key(&[value], &mut arena).unwrap_err().code,
                "protocol"
            );
        }
        let mut exhausted = Arena::new(0);
        assert_eq!(
            symmetry
                .key(&[Some(Scalar::Int(0))], &mut exhausted)
                .unwrap_err()
                .code,
            "budget"
        );
    }
}
