use super::Search;
use crate::CoreError;
use crate::safety::Scalar;
use crate::safety_expressions::{EvalContext, visit_branch};
use crate::safety_values::{Arena, ValueDelta};
use crate::safety_work::WorkBudget;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use std::ops::Range;

enum Event {
    Candidate {
        parent: usize,
        action: usize,
        start: usize,
    },
    Deadlock(usize),
    Error(CoreError),
}

struct Buffer {
    events: Vec<Event>,
    values: Vec<Option<Scalar>>,
    next: Vec<Option<Scalar>>,
}

impl Buffer {
    fn new(width: usize) -> Self {
        Self {
            events: vec![],
            values: vec![],
            next: vec![None; width],
        }
    }
    fn reset(&mut self) {
        self.events.clear();
        self.values.clear();
        self.next.fill(None);
    }
    #[cfg(test)]
    fn candidate(&mut self, parent: usize, action: usize, values: &[Option<Scalar>]) {
        self.events.push(Event::Candidate {
            parent,
            action,
            start: self.values.len(),
        });
        self.values.extend_from_slice(values);
    }
}

pub(super) struct Workers {
    pool: rayon::ThreadPool,
    buffers: Vec<Buffer>,
}

impl Workers {
    pub(super) fn new(workers: usize, width: usize) -> Result<Self, CoreError> {
        Ok(Self {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .map_err(|error| CoreError::protocol(&format!("Safety worker pool: {error}")))?,
            buffers: (0..workers).map(|_| Buffer::new(width)).collect(),
        })
    }
}

impl Search<'_> {
    pub(super) fn parallel_frontier(
        &mut self,
        frontier: Range<usize>,
        workers: &mut Workers,
        arena: &mut Arena,
    ) -> Result<(), CoreError> {
        let mut start = frontier.start;
        while start < frontier.end {
            let chunk = (frontier.end - start)
                .div_ceil(workers.buffers.len())
                .min(64);
            let end = (start + chunk * workers.buffers.len()).min(frontier.end);
            let ranges: Vec<_> = (start..end)
                .step_by(chunk)
                .map(|first| first..(first + chunk).min(end))
                .collect();
            let budget = WorkBudget::new(self.bounds.max_work - arena.work, ranges.len());
            let expansions: Vec<_> = workers.pool.install(|| {
                ranges
                    .into_par_iter()
                    .zip(workers.buffers.par_iter_mut())
                    .enumerate()
                    .map(|(worker, (range, buffer))| {
                        self.expand_range(range, arena, &budget, worker, buffer)
                    })
                    .collect()
            });
            arena.work += expansions.iter().map(|delta| delta.work).sum::<u32>();
            for (delta, buffer) in expansions.into_iter().zip(&mut workers.buffers) {
                let mapping = arena.merge(delta)?;
                for value in buffer.values.iter_mut().flatten() {
                    *value = mapping.get(*value);
                }
                for event in buffer.events.drain(..) {
                    match event {
                        Event::Candidate {
                            parent,
                            action,
                            start,
                        } => {
                            self.accept(
                                &buffer.values[start..start + self.store.width],
                                Some(parent),
                                action,
                                arena,
                            )?;
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
        buffer: &mut Buffer,
    ) -> ValueDelta {
        buffer.reset();
        let mut arena = base.fork(budget, worker);
        for parent in range {
            let current = self.store.state(parent);
            let mut enabled = false;
            for (action, branch) in self.program.next.iter().enumerate() {
                let result = visit_branch(
                    branch,
                    0,
                    EvalContext::new(current, &[]),
                    &mut buffer.next,
                    &mut arena,
                    &mut |values, _arena| {
                        enabled = true;
                        buffer.events.push(Event::Candidate {
                            parent,
                            action,
                            start: buffer.values.len(),
                        });
                        buffer.values.extend_from_slice(values);
                        Ok(false)
                    },
                );
                match result {
                    Ok(true) => return arena.into_delta(),
                    Ok(false) => {}
                    Err(error) => {
                        buffer.events.push(Event::Error(error));
                        return arena.into_delta();
                    }
                }
            }
            if !enabled && self.program.check_deadlock {
                buffer.events.push(Event::Deadlock(parent));
                break;
            }
        }
        arena.into_delta()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_keep_snapshots_and_order_when_buffers_are_reused() {
        let mut buffer = Buffer::new(2);
        let mut values = [Some(Scalar::Int(1)), Some(Scalar::Int(2))];
        buffer.candidate(3, 0, &values);
        values[0] = Some(Scalar::Int(7));
        buffer.candidate(4, 1, &values);
        assert_eq!(
            buffer.values,
            [
                Some(Scalar::Int(1)),
                Some(Scalar::Int(2)),
                Some(Scalar::Int(7)),
                Some(Scalar::Int(2))
            ]
        );
        for (event, expected) in buffer.events.iter().zip([(3, 0, 0), (4, 1, 2)]) {
            let Event::Candidate {
                parent,
                action,
                start,
                ..
            } = event
            else {
                panic!("expected candidate")
            };
            assert_eq!((*parent, *action, *start), expected);
        }
        buffer.reset();
        buffer.candidate(5, 2, &values);
        assert_eq!(buffer.values, values);
        assert_eq!(buffer.events.len(), 1);
    }
}
