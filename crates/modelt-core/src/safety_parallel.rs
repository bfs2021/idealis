use super::{Search, StateStore, check_invariants, get_hash};
use crate::CoreError;
use crate::safety::Scalar;
use crate::safety_expressions::{EvalContext, visit_branch};
use crate::safety_values::{Arena, ValueDelta};
use crate::safety_work::WorkBudget;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::ops::Range;

enum Event {
    Candidate {
        parent: usize,
        action: usize,
        values: Vec<Option<Scalar>>,
        checked: Result<Option<usize>, CoreError>,
    },
    Deadlock(usize),
    Error(CoreError),
}

struct Expansion {
    delta: ValueDelta,
    events: Vec<Event>,
}

impl Search<'_> {
    pub(super) fn parallel_frontier(
        &mut self,
        frontier: Range<usize>,
        pool: &rayon::ThreadPool,
        arena: &mut Arena,
    ) -> Result<(), CoreError> {
        let mut start = frontier.start;
        while start < frontier.end {
            let chunk = (frontier.end - start)
                .div_ceil(pool.current_num_threads())
                .min(64);
            let end = (start + chunk * pool.current_num_threads()).min(frontier.end);
            let ranges: Vec<_> = (start..end)
                .step_by(chunk)
                .map(|first| first..(first + chunk).min(end))
                .collect();
            let budget = WorkBudget::new(self.bounds.max_work - arena.work, ranges.len());
            let expansions: Vec<_> = pool.install(|| {
                ranges
                    .into_par_iter()
                    .enumerate()
                    .map(|(worker, range)| self.expand_range(range, arena, &budget, worker))
                    .collect()
            });
            arena.work += expansions
                .iter()
                .map(|expansion| expansion.delta.work)
                .sum::<u32>();
            for expansion in expansions {
                let mapping = arena.merge(expansion.delta)?;
                for event in expansion.events {
                    match event {
                        Event::Candidate {
                            parent,
                            action,
                            values,
                            checked,
                        } => {
                            let values: Vec<_> = values
                                .into_iter()
                                .map(|value| value.map(|value| mapping.get(value)))
                                .collect();
                            self.accept(&values, Some(parent), action, arena, Some(checked))?;
                        }
                        Event::Deadlock(parent) => self.deadlock(parent, arena)?,
                        Event::Error(error) => return Err(error),
                    }
                    if self.stopped() {
                        return Ok(());
                    }
                }
            }
            start = end;
        }
        Ok(())
    }

    fn expand_range(
        &self,
        range: Range<usize>,
        base: &Arena,
        budget: &WorkBudget,
        worker: usize,
    ) -> Expansion {
        let mut arena = base.fork(budget, worker);
        let mut events = vec![];
        let mut local = StateStore::new(self.store.width);
        let mut next = vec![None; self.store.width];
        for parent in range {
            let current = self.store.state(parent);
            let mut enabled = false;
            for (action, branch) in self.program.next.iter().enumerate() {
                let result = visit_branch(
                    branch,
                    0,
                    EvalContext::new(current, &[]),
                    &mut next,
                    &mut arena,
                    &mut |values, arena| {
                        enabled = true;
                        let hash = get_hash(values)?;
                        let checked = if self.nodes[parent].depth < self.bounds.max_depth
                            && self.store.find(hash, values).is_none()
                            && local.find(hash, values).is_none()
                        {
                            local.push(hash, values)?;
                            check_invariants(
                                self.program,
                                &values.iter().flatten().copied().collect::<Vec<_>>(),
                                arena,
                            )
                        } else {
                            Ok(None)
                        };
                        let stop = !matches!(checked, Ok(None));
                        events.push(Event::Candidate {
                            parent,
                            action,
                            values: values.to_vec(),
                            checked,
                        });
                        Ok(stop)
                    },
                );
                match result {
                    Ok(true) => {
                        return Expansion {
                            delta: arena.into_delta(),
                            events,
                        };
                    }
                    Ok(false) => {}
                    Err(error) => {
                        events.push(Event::Error(error));
                        return Expansion {
                            delta: arena.into_delta(),
                            events,
                        };
                    }
                }
            }
            if !enabled && self.program.check_deadlock {
                events.push(Event::Deadlock(parent));
                break;
            }
        }
        Expansion {
            delta: arena.into_delta(),
            events,
        }
    }
}
