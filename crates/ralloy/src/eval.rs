use crate::{
    Stats,
    syntax::{Binary, Declaration, Expr, Module, Quantifier, Unary},
};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) enum Relation {
    Unary(u64),
    Binary(Vec<u64>),
}
enum Value {
    Formula(bool),
    Relation(Relation),
}
impl Value {
    fn formula(self) -> bool {
        let Self::Formula(value) = self else {
            unreachable!("validated formula")
        };
        value
    }
    fn relation(self) -> Relation {
        let Self::Relation(value) = self else {
            unreachable!("validated relation")
        };
        value
    }
}
impl Relation {
    fn rows(self) -> Vec<u64> {
        match self {
            Self::Binary(rows) => rows,
            _ => unreachable!("validated binary relation"),
        }
    }
    fn set(self) -> u64 {
        match self {
            Self::Unary(set) => set,
            _ => unreachable!("validated unary relation"),
        }
    }
    fn count(&self) -> u32 {
        match self {
            Self::Unary(set) => set.count_ones(),
            Self::Binary(rows) => rows.iter().map(|row| row.count_ones()).sum(),
        }
    }
}
pub(crate) struct Budget {
    pub stats: Stats,
    pub limit: u64,
}
impl Budget {
    pub fn tick(&mut self) -> Result<(), ()> {
        if self.stats.work == self.limit {
            return Err(());
        }
        self.stats.work += 1;
        Ok(())
    }
}
pub(crate) struct World {
    pub names: Vec<String>,
    pub sets: BTreeMap<String, u64>,
    pub fields: Vec<Vec<u64>>,
}
type Environment = BTreeMap<String, Relation>;
pub(crate) struct Evaluator<'a> {
    pub module: &'a Module,
    pub world: &'a World,
    pub budget: &'a mut Budget,
}
impl Evaluator<'_> {
    pub fn formula(&mut self, expression: &Expr) -> Result<bool, ()> {
        self.expression(expression, &Environment::new())
            .map(Value::formula)
    }
    fn expression(&mut self, expression: &Expr, locals: &Environment) -> Result<Value, ()> {
        self.budget.tick()?;
        match expression {
            Expr::Name(name) => {
                if let Some(value) = locals.get(name) {
                    return Ok(Value::Relation(value.clone()));
                }
                if name == "none" {
                    return Ok(Value::Relation(Relation::Unary(0)));
                }
                if name == "iden" {
                    return Ok(Value::Relation(Relation::Binary(
                        (0..self.world.names.len())
                            .map(|index| 1 << index)
                            .collect(),
                    )));
                }
                if let Some(set) = self.world.sets.get(name) {
                    return Ok(Value::Relation(Relation::Unary(*set)));
                }
                if let Some(index) = self
                    .module
                    .fields
                    .iter()
                    .position(|field| &field.name == name)
                {
                    return Ok(Value::Relation(Relation::Binary(
                        self.world.fields[index].clone(),
                    )));
                }
                self.expression(&self.module.predicates[name], &Environment::new())
            }
            Expr::Call(name) => self.expression(&self.module.predicates[name], &Environment::new()),
            Expr::Block(expressions) => {
                for expression in expressions {
                    if !self.expression(expression, locals)?.formula() {
                        return Ok(Value::Formula(false));
                    }
                }
                Ok(Value::Formula(true))
            }
            Expr::Unary(operator, child) => {
                let value = self.expression(child, locals)?;
                if matches!(operator, Unary::Not) {
                    return Ok(Value::Formula(!value.formula()));
                }
                let relation = value.relation();
                if matches!(operator, Unary::No | Unary::Some | Unary::One | Unary::Lone) {
                    let count = relation.count();
                    return Ok(Value::Formula(match operator {
                        Unary::No => count == 0,
                        Unary::Some => count > 0,
                        Unary::One => count == 1,
                        _ => count <= 1,
                    }));
                }
                let mut rows = relation.rows();
                if matches!(operator, Unary::Transpose) {
                    let mut transposed = vec![0; rows.len()];
                    for (source, row) in rows.iter().enumerate() {
                        for (target, result) in transposed.iter_mut().enumerate() {
                            self.budget.tick()?;
                            if row & (1 << target) != 0 {
                                *result |= 1 << source;
                            }
                        }
                    }
                    return Ok(Value::Relation(Relation::Binary(transposed)));
                }
                for middle in 0..rows.len() {
                    let successors = rows[middle];
                    for row in &mut rows {
                        self.budget.tick()?;
                        if *row & (1 << middle) != 0 {
                            *row |= successors;
                        }
                    }
                }
                if matches!(operator, Unary::Reflexive) {
                    for (index, row) in rows.iter_mut().enumerate() {
                        *row |= 1 << index;
                    }
                }
                Ok(Value::Relation(Relation::Binary(rows)))
            }
            Expr::Binary(operator, left, right) => {
                let left = self.expression(left, locals)?;
                if matches!(operator, Binary::And) && !left.formula_ref() {
                    return Ok(Value::Formula(false));
                }
                if matches!(operator, Binary::Or) && left.formula_ref() {
                    return Ok(Value::Formula(true));
                }
                if matches!(operator, Binary::Implies) && !left.formula_ref() {
                    return Ok(Value::Formula(true));
                }
                let right = self.expression(right, locals)?;
                if matches!(
                    operator,
                    Binary::And | Binary::Or | Binary::Implies | Binary::Iff
                ) {
                    let left = left.formula();
                    let right = right.formula();
                    return Ok(Value::Formula(match operator {
                        Binary::And => left && right,
                        Binary::Or => left || right,
                        Binary::Implies => !left || right,
                        _ => left == right,
                    }));
                }
                let left = left.relation();
                let right = right.relation();
                self.relations(*operator, left, right)
            }
            Expr::Quantified(quantifier, declarations, body) => {
                let mut matched = 0;
                self.quantify(*quantifier, declarations, 0, body, locals, &mut matched)?;
                Ok(Value::Formula(match quantifier {
                    Quantifier::All | Quantifier::No => matched == 0,
                    Quantifier::Some => matched > 0,
                    Quantifier::One => matched == 1,
                    Quantifier::Lone => matched <= 1,
                }))
            }
        }
    }
    fn relations(
        &mut self,
        operator: Binary,
        left: Relation,
        right: Relation,
    ) -> Result<Value, ()> {
        let count = self.world.names.len();
        if matches!(operator, Binary::Product) {
            let left = left.set();
            let right = right.set();
            return Ok(Value::Relation(Relation::Binary(
                (0..count)
                    .map(|index| if left & (1 << index) != 0 { right } else { 0 })
                    .collect(),
            )));
        }
        if matches!(operator, Binary::Domain | Binary::Range) {
            let (set, mut rows) = if matches!(operator, Binary::Domain) {
                (left.set(), right.rows())
            } else {
                (right.set(), left.rows())
            };
            for (index, row) in rows.iter_mut().enumerate() {
                self.budget.tick()?;
                if matches!(operator, Binary::Domain) {
                    if set & (1 << index) == 0 {
                        *row = 0;
                    }
                } else {
                    *row &= set;
                }
            }
            return Ok(Value::Relation(Relation::Binary(rows)));
        }
        if matches!(operator, Binary::Join) {
            let result = match (left, right) {
                (Relation::Unary(set), Relation::Binary(rows)) => {
                    let mut output = 0;
                    for (index, row) in rows.into_iter().enumerate() {
                        self.budget.tick()?;
                        if set & (1 << index) != 0 {
                            output |= row;
                        }
                    }
                    Relation::Unary(output)
                }
                (Relation::Binary(rows), Relation::Unary(set)) => {
                    let mut output = 0;
                    for (index, row) in rows.into_iter().enumerate() {
                        self.budget.tick()?;
                        if row & set != 0 {
                            output |= 1 << index;
                        }
                    }
                    Relation::Unary(output)
                }
                (Relation::Binary(left), Relation::Binary(right)) => {
                    let mut output = vec![0; count];
                    for (source, row) in left.into_iter().enumerate() {
                        for (middle, successors) in right.iter().enumerate() {
                            self.budget.tick()?;
                            if row & (1 << middle) != 0 {
                                output[source] |= successors;
                            }
                        }
                    }
                    Relation::Binary(output)
                }
                _ => unreachable!("validated join"),
            };
            return Ok(Value::Relation(result));
        }
        let binary = matches!(left, Relation::Binary(_)) || matches!(right, Relation::Binary(_));
        let (left, right) = if binary {
            (left.rows(), right.rows())
        } else {
            (vec![left.set()], vec![right.set()])
        };
        if matches!(
            operator,
            Binary::Equal | Binary::NotEqual | Binary::In | Binary::NotIn
        ) {
            let subset = matches!(operator, Binary::In | Binary::NotIn);
            let mut satisfied = true;
            for (left, right) in left.iter().zip(&right) {
                self.budget.tick()?;
                satisfied &= if subset {
                    left & !right == 0
                } else {
                    left == right
                };
            }
            return Ok(Value::Formula(
                satisfied != matches!(operator, Binary::NotEqual | Binary::NotIn),
            ));
        }
        let mut output = Vec::new();
        for (left, right) in left.into_iter().zip(right) {
            self.budget.tick()?;
            output.push(match operator {
                Binary::Union => left | right,
                Binary::Difference => left & !right,
                Binary::Intersection => left & right,
                _ => unreachable!("validated relational operator"),
            });
        }
        Ok(Value::Relation(if binary {
            Relation::Binary(output)
        } else {
            Relation::Unary(output[0])
        }))
    }
    fn quantify(
        &mut self,
        quantifier: Quantifier,
        declarations: &[Declaration],
        index: usize,
        body: &Expr,
        locals: &Environment,
        matched: &mut u64,
    ) -> Result<bool, ()> {
        self.budget.tick()?;
        if index == declarations.len() {
            let value = self.expression(body, locals)?.formula();
            if value != matches!(quantifier, Quantifier::All) {
                *matched += 1;
            }
            return Ok(match quantifier {
                Quantifier::All | Quantifier::No | Quantifier::Some => *matched > 0,
                Quantifier::One | Quantifier::Lone => *matched > 1,
            });
        }
        let declaration = &declarations[index];
        let domain = self
            .expression(&declaration.domain, locals)?
            .relation()
            .set();
        self.bind(
            quantifier,
            declarations,
            index,
            0,
            domain,
            0,
            body,
            locals,
            matched,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn bind(
        &mut self,
        quantifier: Quantifier,
        declarations: &[Declaration],
        declaration_index: usize,
        name_index: usize,
        domain: u64,
        used: u64,
        body: &Expr,
        locals: &Environment,
        matched: &mut u64,
    ) -> Result<bool, ()> {
        let declaration = &declarations[declaration_index];
        if name_index == declaration.names.len() {
            return self.quantify(
                quantifier,
                declarations,
                declaration_index + 1,
                body,
                locals,
                matched,
            );
        }
        let mut choices = if declaration.disjoint {
            domain & !used
        } else {
            domain
        };
        while choices != 0 {
            self.budget.tick()?;
            let atom = choices.isolate_lowest_one();
            choices &= !atom;
            let mut locals = locals.clone();
            locals.insert(declaration.names[name_index].clone(), Relation::Unary(atom));
            if self.bind(
                quantifier,
                declarations,
                declaration_index,
                name_index + 1,
                domain,
                used | atom,
                body,
                &locals,
                matched,
            )? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
impl Value {
    fn formula_ref(&self) -> bool {
        let Self::Formula(value) = self else {
            unreachable!("validated formula")
        };
        *value
    }
}
