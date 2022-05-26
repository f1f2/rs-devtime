use std::collections::VecDeque;
use std::ops::Add;
use std::time::{Duration, Instant, SystemTime};

pub(crate) trait Signal {
    fn signal(&self);
}

pub(crate) struct Sleeper<S: Signal> {
    pub(crate) monotonic: Instant,
    pub(crate) signal: S,
}

pub(crate) struct SleepWaiter<S: Signal> {
    pub(crate) count: usize,
    pub(crate) signal: S,
}

pub(crate) struct FakeTimeState<S: Signal> {
    monotonic: Instant,
    real: SystemTime,
    sleepers: VecDeque<Sleeper<S>>,
    sleep_waiters: Vec<SleepWaiter<S>>,
}

impl<S: Signal> FakeTimeState<S> {
    pub(crate) fn monotonic(&self) -> Instant {
        return self.monotonic;
    }

    pub(crate) fn real(&self) -> SystemTime {
        return self.real;
    }

    pub(crate) fn add_sleeper(&mut self, waiter: Sleeper<S>) {
        match self
            .sleepers
            .binary_search_by(|item| item.monotonic.cmp(&waiter.monotonic))
        {
            Ok(index) | Err(index) => self.sleepers.insert(index, waiter),
        };
        self.fire_sleep_waiters();
        self.fire_sleepers();
    }

    pub(crate) fn add_sleep_waiter(&mut self, waiter: SleepWaiter<S>) {
        self.sleep_waiters.push(waiter);
        self.fire_sleep_waiters();
    }

    pub(crate) fn advance(&mut self, duration: Duration) {
        self.monotonic = self.monotonic.add(duration);
        self.real = self.real.add(duration);

        self.fire_sleepers();
    }

    fn fire_sleepers(&mut self) {
        loop {
            if let Some(earliest) = self.sleepers.front() {
                if earliest.monotonic <= self.monotonic {
                    earliest.signal.signal();
                    self.sleepers.pop_front();
                    self.fire_sleep_waiters();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn fire_sleep_waiters(&mut self) {
        let sleepers_count = self.sleepers.len();
        let waiters_count = self.sleep_waiters.len();
        for k in 0..waiters_count {
            let index = waiters_count - k - 1;
            if self.sleep_waiters[index].count == sleepers_count {
                let waiter = self.sleep_waiters.remove(index);
                waiter.signal.signal();
            }
        }
    }
}

impl<S: Signal> Default for FakeTimeState<S> {
    fn default() -> Self {
        return FakeTimeState {
            monotonic: Instant::now(),
            real: SystemTime::UNIX_EPOCH,
            sleepers: Default::default(),
            sleep_waiters: Default::default(),
        };
    }
}
