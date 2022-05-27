use devtime::sync::RealTime;
use std::time::Duration;

fn pause<T: devtime::sync::Time>(t: T, duration: Duration) {
    t.sleep(duration)
}

fn main() {
    let clock = RealTime::default();
    pause(clock, Duration::from_secs(1))
}

#[cfg(test)]
mod test {
    use super::*;
    use devtime::sync::{FakeTime, Time};
    use std::ops::Add;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::thread::sleep;

    #[test]
    fn test_pause() {
        let clock = FakeTime::default();
        let t_clock = clock.clone();
        let start = (clock.monotonic_now(), clock.system_now());
        let now = Arc::new(Mutex::new((clock.monotonic_now(), clock.system_now())));
        let now2 = now.clone();
        let handler = thread::spawn(move || {
            pause(t_clock.clone(), Duration::from_secs(1));
            let mon_now = t_clock.monotonic_now();
            let sys_now = t_clock.system_now();
            *now2.lock().unwrap() = (mon_now, sys_now);
        });

        for _ in 0..15 {
            sleep(Duration::from_millis(10));
            clock.advance(Duration::from_millis(100));
        }
        let res = handler.join();
        assert!(res.is_ok());
        assert_eq!(
            (
                start.0.add(Duration::from_secs(1)),
                start.1.add(Duration::from_secs(1))
            ),
            *now.lock().unwrap()
        );
    }
}
