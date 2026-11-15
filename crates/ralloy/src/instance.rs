use crate::eval::{Budget, Evaluator, World};
use crate::syntax::{Expr, Module, Mult};
use crate::{CheckReason, CheckStatus, Command, Error, ErrorKind, Instance, InstanceCheck, Stats};
use std::collections::BTreeMap;

enum Failure {
    Invalid(CheckReason),
    Mismatch,
    Unknown,
    Protocol(&'static str),
}
impl From<()> for Failure {
    fn from(_: ()) -> Self {
        Self::Unknown
    }
}

pub(crate) fn check(
    module: &Module,
    command: &Command,
    body: &Expr,
    instance: &Instance,
    max_work: u64,
) -> Result<InstanceCheck, Error> {
    let mut budget = Budget {
        stats: Stats::default(),
        limit: max_work,
    };
    let (status, reason) = match evaluate(module, command, body, instance, &mut budget) {
        Ok(()) => (CheckStatus::Matched, None),
        Err(Failure::Invalid(reason)) => (CheckStatus::Invalid, Some(reason)),
        Err(Failure::Mismatch) => (CheckStatus::Mismatch, Some(CheckReason::Property)),
        Err(Failure::Unknown) => (CheckStatus::Unknown, Some(CheckReason::Work)),
        Err(Failure::Protocol(message)) => return Err(Error::new(ErrorKind::Protocol, message)),
    };
    Ok(InstanceCheck {
        command: command.clone(),
        status,
        reason,
        stats: budget.stats,
    })
}

fn evaluate(
    module: &Module,
    command: &Command,
    body: &Expr,
    instance: &Instance,
    budget: &mut Budget,
) -> Result<(), Failure> {
    if instance.signatures.len() != module.signatures.len()
        || module
            .signatures
            .iter()
            .any(|signature| !instance.signatures.contains_key(&signature.name))
        || instance.fields.len() != module.fields.len()
        || module
            .fields
            .iter()
            .any(|field| !instance.fields.contains_key(&field.name))
    {
        return Err(Failure::Protocol(
            "instance must specify exactly the declared signatures and fields",
        ));
    }
    let mut world = World::new(command.scope.integer_bitwidth);
    let mut atoms: BTreeMap<_, _> = world
        .names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect();
    for (signature, bound) in module.signatures.iter().zip(&command.scope.signatures) {
        budget.tick()?;
        let values = &instance.signatures[&signature.name];
        if values.len() > bound.maximum || bound.exact && values.len() != bound.maximum {
            return Err(Failure::Invalid(CheckReason::Scope));
        }
        if signature.mult == Mult::One && values.len() != 1
            || signature.mult == Mult::Some && values.is_empty()
            || signature.mult == Mult::Lone && values.len() > 1
        {
            return Err(Failure::Invalid(CheckReason::Declaration));
        }
        let mut set = 0;
        for name in values {
            budget.tick()?;
            if name.is_empty()
                || name.len() > 256
                || atoms.insert(name.clone(), world.names.len()).is_some()
            {
                return Err(Failure::Protocol(
                    "atom names must be nonempty, unique and at most 256 bytes",
                ));
            }
            set |= 1u64 << world.names.len();
            world.names.push(name.clone());
        }
        world.sets.insert(signature.name.clone(), set);
    }
    world.sets.insert(
        "univ".into(),
        if world.names.len() == 64 {
            u64::MAX
        } else {
            (1u64 << world.names.len()) - 1
        },
    );
    for field in &module.fields {
        let mut rows = vec![0u64; world.names.len()];
        for [source, target] in &instance.fields[&field.name] {
            budget.tick()?;
            let source = *atoms
                .get(source)
                .ok_or(Failure::Protocol("unknown field source atom"))?;
            let target = *atoms
                .get(target)
                .ok_or(Failure::Protocol("unknown field target atom"))?;
            if world.sets[&field.owner] & (1 << source) == 0
                || world.sets[&field.target] & (1 << target) == 0
            {
                return Err(Failure::Invalid(CheckReason::Declaration));
            }
            if rows[source] & (1 << target) != 0 {
                return Err(Failure::Protocol("duplicate field tuple"));
            }
            rows[source] |= 1 << target;
        }
        for (source, row) in rows.iter().enumerate() {
            budget.tick()?;
            if world.sets[&field.owner] & (1 << source) != 0 {
                let count = row.count_ones();
                if field.mult == Mult::One && count != 1
                    || field.mult == Mult::Lone && count > 1
                    || field.mult == Mult::Some && count == 0
                {
                    return Err(Failure::Invalid(CheckReason::Declaration));
                }
            }
        }
        world.fields.push(rows);
    }
    budget.stats.candidates = 1;
    let mut evaluator = Evaluator {
        module,
        world: &world,
        budget,
    };
    for fact in &module.facts {
        if !evaluator.formula(fact)? {
            return Err(Failure::Invalid(CheckReason::Facts));
        }
    }
    evaluator.budget.stats.fact_instances = 1;
    if !evaluator.formula(body)? {
        return Err(Failure::Mismatch);
    }
    Ok(())
}
