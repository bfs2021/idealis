use crate::eval::{Budget, Evaluator, World};
use crate::syntax::{Expr, Module, Mult};
use crate::{Analysis, Command, Error, Instance, Kind, Stats, Status};

pub(crate) fn analyze(
    module: &Module,
    command: &Command,
    body: &Expr,
    max_work: u64,
) -> Result<Analysis, Error> {
    let mut search = Search {
        module,
        command,
        body,
        budget: Budget {
            stats: Stats::default(),
            limit: max_work,
        },
        found: None,
    };
    let exhausted = search.counts(0, &mut Vec::new());
    let status = match exhausted {
        Err(()) => Status::Unknown,
        Ok(true) if command.kind == Kind::Run => Status::Instance,
        Ok(true) => Status::Counterexample,
        Ok(false) if command.kind == Kind::Check && search.budget.stats.fact_instances > 0 => {
            Status::NoCounterexample
        }
        Ok(false) => Status::NoInstance,
    };
    Ok(Analysis {
        command: command.clone(),
        status,
        instance: search.found,
        stats: search.budget.stats,
    })
}
struct Search<'a> {
    module: &'a Module,
    command: &'a Command,
    body: &'a Expr,
    budget: Budget,
    found: Option<Instance>,
}
struct Slot {
    field: usize,
    source: usize,
    target: u64,
    mult: Mult,
}

impl Search<'_> {
    fn counts(&mut self, index: usize, counts: &mut Vec<usize>) -> Result<bool, ()> {
        self.budget.tick()?;
        if index < self.module.signatures.len() {
            let bound = &self.command.scope.signatures[index];
            let minimum = if bound.exact {
                bound.maximum
            } else if self.module.signatures[index].mult == Mult::Some {
                1
            } else {
                0
            };
            let maximum = bound.maximum;
            for count in minimum..=maximum {
                counts.push(count);
                let result = self.counts(index + 1, counts)?;
                counts.pop();
                if result {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        let World {
            mut names,
            mut sets,
            ..
        } = World::new(self.command.scope.integer_bitwidth);
        for (signature, count) in self.module.signatures.iter().zip(counts.iter()) {
            let mut set = 0;
            for index in 0..*count {
                set |= 1u64 << names.len();
                names.push(format!("{}${index}", signature.name));
            }
            sets.insert(signature.name.clone(), set);
        }
        sets.insert(
            "univ".into(),
            if names.len() == 64 {
                u64::MAX
            } else {
                (1u64 << names.len()) - 1
            },
        );
        let mut slots = Vec::new();
        for (field_index, field) in self.module.fields.iter().enumerate() {
            let mut owners = sets[&field.owner];
            while owners != 0 {
                let source = owners.trailing_zeros() as usize;
                owners &= owners - 1;
                slots.push(Slot {
                    field: field_index,
                    source,
                    target: sets[&field.target],
                    mult: field.mult,
                });
            }
        }
        let fields = vec![vec![0; names.len()]; self.module.fields.len()];
        self.assign(
            &slots,
            0,
            &mut World {
                names,
                sets,
                fields,
            },
        )
    }
    fn assign(&mut self, slots: &[Slot], index: usize, world: &mut World) -> Result<bool, ()> {
        self.budget.tick()?;
        if index == slots.len() {
            return self.candidate(world);
        }
        let slot = &slots[index];
        if matches!(slot.mult, Mult::One | Mult::Lone) {
            if slot.mult == Mult::Lone {
                world.fields[slot.field][slot.source] = 0;
                if self.assign(slots, index + 1, world)? {
                    return Ok(true);
                }
            }
            let mut choices = slot.target;
            while choices != 0 {
                self.budget.tick()?;
                let choice = choices.isolate_lowest_one();
                choices &= !choice;
                world.fields[slot.field][slot.source] = choice;
                if self.assign(slots, index + 1, world)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        let mut choice = 0;
        loop {
            self.budget.tick()?;
            if slot.mult == Mult::Set || choice != 0 {
                world.fields[slot.field][slot.source] = choice;
                if self.assign(slots, index + 1, world)? {
                    return Ok(true);
                }
            }
            choice = choice.wrapping_sub(slot.target) & slot.target;
            if choice == 0 {
                return Ok(false);
            }
        }
    }
    fn candidate(&mut self, world: &World) -> Result<bool, ()> {
        self.budget.tick()?;
        self.budget.stats.candidates += 1;
        let mut evaluator = Evaluator {
            module: self.module,
            world,
            budget: &mut self.budget,
        };
        for fact in &self.module.facts {
            if !evaluator.formula(fact)? {
                return Ok(false);
            }
        }
        evaluator.budget.stats.fact_instances += 1;
        let holds = evaluator.formula(self.body)?;
        if holds != (self.command.kind == Kind::Run) {
            return Ok(false);
        }
        let signatures = self
            .module
            .signatures
            .iter()
            .map(|signature| {
                let atoms = world
                    .names
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| world.sets[&signature.name] & (1 << index) != 0)
                    .map(|(_, name)| name.clone())
                    .collect();
                (signature.name.clone(), atoms)
            })
            .collect();
        let fields = self
            .module
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let mut tuples = Vec::new();
                for (source, row) in world.fields[index].iter().enumerate() {
                    let mut targets = *row;
                    while targets != 0 {
                        let target = targets.trailing_zeros() as usize;
                        targets &= targets - 1;
                        tuples.push([world.names[source].clone(), world.names[target].clone()]);
                    }
                }
                (field.name.clone(), tuples)
            })
            .collect();
        self.found = Some(Instance { signatures, fields });
        Ok(true)
    }
}
