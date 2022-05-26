use std::collections::VecDeque;
use std::ops::Add;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

trait Time {
    fn monotonic_now(&self) -> Instant;
    fn system_now(&self) -> SystemTime;
    fn sleep(&self, duration: Duration);
    fn sleep_until(&self, dst: Instant);
}

pub struct RealTime {}

impl Time for RealTime {
    fn monotonic_now(&self) -> Instant {
        Instant::now()
    }

    fn system_now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }

    fn sleep_until(&self, dst: Instant) {
        thread::sleep(dst.saturating_duration_since(Instant::now()));
    }
}

struct FakeTimeState {
    monotonic: Instant,
    real: SystemTime,
    waiters: VecDeque<Sleeper>,
}

impl FakeTimeState {
    fn add_waiter(&mut self, waiter: Sleeper) {
        match self
            .waiters
            .binary_search_by(|item| item.monotonic.cmp(&waiter.monotonic))
        {
            Ok(index) | Err(index) => self.waiters.insert(index, waiter),
        }
    }
}

struct Sleeper {
    monotonic: Instant,
    chan: Sender<()>,
}

#[derive(Clone)]
pub struct FakeTime(Arc<Mutex<FakeTimeState>>);

impl FakeTime {
    fn advance(&self, duration: Duration) {
        let mut state = self.0.lock().unwrap();
        let now = state.monotonic.add(duration);
        state.monotonic = now;
        state.real.add(duration);

        loop {
            if let Some(earliest) = state.waiters.front() {
                if earliest.monotonic <= now {
                    // earliest.chan.send(()).ok();
                    state.waiters.pop_front();
                }
            } else {
                break;
            }
        }
    }
}

impl Time for FakeTime {
    fn monotonic_now(&self) -> Instant {
        return self.0.lock().unwrap().monotonic;
    }

    fn system_now(&self) -> SystemTime {
        return self.0.lock().unwrap().real;
    }

    fn sleep(&self, duration: Duration) {
        self.sleep_until(self.monotonic_now().add(duration))
    }

    fn sleep_until(&self, dst: Instant) {
        let (tx, rx) = mpsc::channel::<()>();
        {
            let mut state = self.0.lock().unwrap();

            if state.monotonic >= dst {
                return;
            }

            let waiter = Sleeper {
                monotonic: dst,
                chan: tx,
            };
            state.add_waiter(waiter);
        }
        rx.recv().unwrap();
    }
}

impl Default for FakeTime {
    fn default() -> Self {
        return FakeTime(Arc::new(Mutex::new(FakeTimeState {
            monotonic: Instant::now(),
            real: SystemTime::UNIX_EPOCH,
            waiters: VecDeque::new(),
        })));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sleep() {
        let clock = FakeTime::default();
        let dst = clock.monotonic_now().add(Duration::from_secs(60));
        let clock2 = clock.clone();
        let handler = thread::spawn(move || {
            clock2.sleep_until(dst);
            assert_eq!(dst, clock2.monotonic_now());
        });

        clock.advance(Duration::from_secs(60));
        handler.join().unwrap();
    }
}
