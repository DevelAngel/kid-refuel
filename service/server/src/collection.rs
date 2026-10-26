use std::future::Future;
use std::time::Duration;

use rand::Rng;
use tokio::sync::watch;
use tokio::time;
use tracing::error;

use chrono::{NaiveTime, Timelike, Utc};
use chrono_tz::Tz;

const NIGHT_INTERVAL: Duration = Duration::from_secs(3 * 60 * 60);

pub(crate) fn next_delay<R: Rng>(rng: &mut R, interval: Duration, jitter: Duration) -> Duration {
    let jitter_seconds = jitter.as_secs();
    let interval_seconds = interval.as_secs();
    let minimum = interval_seconds.saturating_sub(jitter_seconds);
    let maximum = interval_seconds.saturating_add(jitter_seconds);

    Duration::from_secs(rng.gen_range(minimum..=maximum))
}

fn interval_at(time: NaiveTime) -> Option<Duration> {
    (time.hour() >= 22 || time.hour() < 2).then_some(NIGHT_INTERVAL)
}

pub(crate) async fn run_schedule<F, Fut, E>(
    mut collect: F,
    mut shutdown: watch::Receiver<bool>,
    interval: Duration,
    jitter: Duration,
    source_time_zone: Tz,
) where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    loop {
        let interval = interval_at(Utc::now().with_timezone(&source_time_zone).time()).unwrap_or(interval);
        let delay = next_delay(&mut rand::thread_rng(), interval, jitter);
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            _ = time::sleep(delay) => {}
        }

        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            result = collect() => {
                if let Err(error) = result {
                    error!("scheduled collection failed: {error}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NIGHT_INTERVAL, interval_at, next_delay};
    use chrono::NaiveTime;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use std::time::Duration;

    #[test]
    fn overnight_interval_is_longer_only_during_quiet_hours() {
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(21, 59, 0).unwrap()),
            None
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(1, 59, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(interval_at(NaiveTime::from_hms_opt(2, 0, 0).unwrap()), None);
    }

    #[test]
    fn first_issue_example_keeps_daytime_jitter_and_thins_the_night() {
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(21, 33, 0).unwrap()),
            None
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(22, 27, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(1, 27, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(4, 27, 0).unwrap()),
            None
        );
    }

    #[test]
    fn second_issue_example_keeps_daytime_jitter_and_thins_the_night() {
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(20, 46, 0).unwrap()),
            None
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(21, 39, 0).unwrap()),
            None
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(22, 39, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(1, 39, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(4, 39, 0).unwrap()),
            None
        );
    }

    #[test]
    fn third_issue_example_keeps_daytime_jitter_and_thins_the_night() {
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(20, 54, 0).unwrap()),
            None
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(22, 4, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(
            interval_at(NaiveTime::from_hms_opt(1, 4, 0).unwrap()),
            Some(NIGHT_INTERVAL)
        );
        assert_eq!(interval_at(NaiveTime::from_hms_opt(4, 4, 0).unwrap()), None);
    }
    #[test]
    fn delay_stays_within_interval_and_jitter_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        let interval = Duration::from_secs(60 * 60);
        let jitter = Duration::from_secs(10 * 60);

        let mut observed = std::collections::BTreeSet::new();
        for _ in 0..1_000 {
            let delay = next_delay(&mut rng, interval, jitter);
            assert!((50 * 60..=70 * 60).contains(&delay.as_secs()));
            observed.insert(delay.as_secs());
        }
        assert!(observed.len() > 1);
    }

    #[tokio::test]
    async fn schedule_continues_after_failure_without_overlapping_runs() {
        use super::run_schedule;
        use std::sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        };
        use tokio::sync::watch;
        use tokio::time::sleep;

        let calls = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicBool::new(false));
        let overlapped = Arc::new(AtomicBool::new(false));
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let schedule = tokio::spawn({
            let calls = Arc::clone(&calls);
            let active = Arc::clone(&active);
            let overlapped = Arc::clone(&overlapped);
            async move {
                run_schedule(
                    move || {
                        let calls = Arc::clone(&calls);
                        let active = Arc::clone(&active);
                        let overlapped = Arc::clone(&overlapped);
                        async move {
                            if active.swap(true, Ordering::SeqCst) {
                                overlapped.store(true, Ordering::SeqCst);
                            }
                            let call = calls.fetch_add(1, Ordering::SeqCst);
                            sleep(Duration::from_millis(2)).await;
                            active.store(false, Ordering::SeqCst);
                            if call == 0 {
                                Err("simulated failure")
                            } else {
                                Ok(())
                            }
                        }
                    },
                    shutdown_rx,
                    Duration::from_millis(1),
                    Duration::ZERO,
                    chrono_tz::Europe::Berlin,
                )
                .await;
            }
        });

        sleep(Duration::from_millis(30)).await;
        let _ = shutdown_tx.send(true);
        schedule.await.expect("schedule should stop cleanly");

        assert!(calls.load(Ordering::SeqCst) >= 2);
        assert!(!overlapped.load(Ordering::SeqCst));
    }
}
