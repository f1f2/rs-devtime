use crate::state::{FakeTimeState, SleepWaiter, Sleeper};
use std::ops::Add;
use std::sync::{mpsc, Arc, Mutex};
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

#[derive(Clone)]
pub struct FakeTime(Arc<Mutex<FakeTimeState>>);

impl FakeTime {
    pub fn advance(&self, d: Duration) {
        self.0.lock().unwrap().advance(d);
    }
    pub fn wait_exact_sleepers_count(&self, count: usize) {
        let (tx, rx) = mpsc::channel();
        let waiter = SleepWaiter { count, chan: tx };
        self.0.lock().unwrap().add_sleep_waiter(waiter);
        rx.recv().unwrap();
    }
}

impl Time for FakeTime {
    fn monotonic_now(&self) -> Instant {
        return self.0.lock().unwrap().monotonic();
    }

    fn system_now(&self) -> SystemTime {
        return self.0.lock().unwrap().real();
    }

    fn sleep(&self, duration: Duration) {
        self.sleep_until(self.monotonic_now().add(duration))
    }

    fn sleep_until(&self, dst: Instant) {
        let (tx, rx) = mpsc::channel::<()>();
        {
            let sleeper = Sleeper {
                monotonic: dst,
                chan: tx,
            };

            self.0.lock().unwrap().add_sleeper(sleeper);
        }
        rx.recv().unwrap();
    }
}

impl Default for FakeTime {
    fn default() -> Self {
        return FakeTime(Arc::new(Mutex::new(FakeTimeState::default())));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_sleep() {
        let clock = FakeTime::default();
        let dst = clock.monotonic_now().add(Duration::from_secs(60));
        let clock2 = clock.clone();
        let thread_worked = Arc::new(AtomicBool::new(false));
        let thread_worked2 = thread_worked.clone();
        let handler = thread::spawn(move || {
            clock2.sleep_until(dst);
            assert_eq!(dst, clock2.monotonic_now());
            thread_worked2.store(true, Ordering::Relaxed);
        });

        clock.advance(Duration::from_secs(60));
        handler.join().unwrap();
        assert_eq!(true, thread_worked.load(Relaxed));
    }

    #[test]
    fn test_wait_sleepers() {
        let clock = FakeTime::default();
        let clock2 = clock.clone();
        let dst = clock.monotonic_now().add(Duration::from_secs(60));
        let thread_worked = Arc::new(AtomicBool::new(false));
        let thread_worked2 = thread_worked.clone();
        let handler = thread::spawn(move || {
            clock2.sleep(Duration::from_secs(60));
            assert_eq!(dst, clock2.monotonic_now());
            thread_worked2.store(true, Ordering::Relaxed);
        });
        clock.wait_exact_sleepers_count(1);
        clock.advance(Duration::from_secs(60));
        handler.join().unwrap();
        assert_eq!(true, thread_worked.load(Relaxed));
    }
}
