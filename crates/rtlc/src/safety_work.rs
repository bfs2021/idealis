use crate::CoreError;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

#[repr(align(128))]
struct WorkCredit(AtomicU32);

pub(crate) struct WorkBudget {
    available: Mutex<u32>,
    credits: Vec<WorkCredit>,
}

impl WorkBudget {
    pub fn new(maximum: u32, workers: usize) -> Self {
        Self {
            available: Mutex::new(maximum),
            credits: (0..workers)
                .map(|_| WorkCredit(AtomicU32::new(0)))
                .collect(),
        }
    }
    pub fn consume(&self, worker: usize, mut count: usize) -> Result<u32, CoreError> {
        let mut consumed = 0;
        while count > 0 {
            let used = take(&self.credits[worker].0, count);
            consumed += used;
            count -= used as usize;
            if count > 0 && !self.refill(worker, count)? {
                break;
            }
        }
        Ok(consumed)
    }
    fn refill(&self, worker: usize, count: usize) -> Result<bool, CoreError> {
        let mut available = self
            .available
            .lock()
            .map_err(|_| CoreError::protocol("Safety work budget lock poisoned"))?;
        let requested = count.max(256);
        let credit = &self.credits[worker].0;
        if credit.load(Ordering::Relaxed) > 0 {
            return Ok(true);
        }
        if *available > 0 {
            let grant = requested.min(*available as usize) as u32;
            *available -= grant;
            credit.fetch_add(grant, Ordering::Relaxed);
            return Ok(true);
        }
        for offset in 1..self.credits.len() {
            let index = (worker + offset) % self.credits.len();
            let grant = take(&self.credits[index].0, requested);
            if grant > 0 {
                credit.fetch_add(grant, Ordering::Relaxed);
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn take(credit: &AtomicU32, count: usize) -> u32 {
    let mut available = credit.load(Ordering::Relaxed);
    while available > 0 {
        let used = count.min(available as usize) as u32;
        match credit.compare_exchange_weak(
            available,
            available - used,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return used,
            Err(actual) => available = actual,
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::WorkBudget;

    #[test]
    fn another_worker_can_spend_unused_credit_below_the_reservation_size() {
        let budget = WorkBudget::new(3, 2);
        assert_eq!(budget.consume(0, 1).unwrap(), 1);
        assert_eq!(budget.consume(1, 2).unwrap(), 2);
        assert_eq!(budget.consume(0, 1).unwrap(), 0);
    }

    #[test]
    fn concurrent_workers_consume_exactly_the_global_budget() {
        for maximum in [1, 7, 255, 256, 257, 100_003] {
            let budget = WorkBudget::new(maximum, 8);
            let consumed: u32 = std::thread::scope(|scope| {
                let handles: Vec<_> = (0..8)
                    .map(|index| {
                        let budget = &budget;
                        scope.spawn(move || {
                            let mut consumed = 0;
                            loop {
                                let count = budget.consume(index, 7).unwrap();
                                consumed += count;
                                if count < 7 {
                                    return consumed;
                                }
                            }
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|handle| handle.join().unwrap())
                    .sum()
            });
            assert_eq!(consumed, maximum);
            assert_eq!(budget.consume(0, 1).unwrap(), 0);
        }
    }

    #[test]
    fn oversized_consumption_returns_the_actual_remaining_work() {
        let budget = WorkBudget::new(7, 2);
        assert_eq!(budget.consume(0, 1).unwrap(), 1);
        assert_eq!(budget.consume(1, usize::MAX).unwrap(), 6);
        assert_eq!(budget.consume(0, 1).unwrap(), 0);
    }
}
