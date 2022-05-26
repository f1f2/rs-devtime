use std::collections::VecDeque;
use std::ops::Add;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant, SystemTime};

pub(crate) struct Sleeper {
    pub(crate) monotonic: Instant,
    pub(crate) chan: Sender<()>,
}

pub(crate) struct SleepWaiter {
    pub(crate) count: usize,
    pub(crate) chan: Sender<()>,
}

pub(crate) struct FakeTimeState {
    monotonic: Instant,
    real: SystemTime,
    sleepers: VecDeque<Sleeper>,
    sleep_waiters: Vec<SleepWaiter>,
}

impl FakeTimeState {
    pub(crate) fn monotonic(&self) -> Instant {
        return self.monotonic;
    }

    pub(crate) fn real(&self) -> SystemTime {
        return self.real;
    }

    pub(crate) fn add_sleeper(&mut self, waiter: Sleeper) {
        match self
            .sleepers
            .binary_search_by(|item| item.monotonic.cmp(&waiter.monotonic))
        {
            Ok(index) | Err(index) => self.sleepers.insert(index, waiter),
        };
        self.fire_sleep_waiters();
        self.fire_sleepers();
    }

    pub(crate) fn add_sleep_waiter(&mut self, waiter: SleepWaiter) {
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
                    earliest.chan.send(()).ok();
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
                let _ = waiter.chan.send(());
            }
        }
    }
}

impl Default for FakeTimeState {
    fn default() -> Self {
        return FakeTimeState {
            monotonic: Instant::now(),
            real: SystemTime::UNIX_EPOCH,
            sleepers: Default::default(),
            sleep_waiters: Default::default(),
        };
    }
}
