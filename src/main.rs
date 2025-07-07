use governor::{DefaultKeyedRateLimiter, Quota};
use std::{
    net::IpAddr,
    num::NonZeroU32,
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::{task, time::sleep};

#[tokio::main]
async fn main() {
    let limiter = Arc::new(DefaultKeyedRateLimiter::<IpAddr>::keyed(Quota::per_minute(
        NonZeroU32::new(u32::try_from(8).unwrap()).unwrap(),
    )));

    let accepted = Arc::new(Mutex::new(0));
    let rejected = Arc::new(Mutex::new(0));

    let start = Instant::now();
    let run_duration = Duration::from_secs(120);
    let calls_per_sec = 200;

    // IP to simulate
    let ip = IpAddr::from_str("192.168.1.100").unwrap();

    while Instant::now() - start < run_duration {
        let mut handles = vec![];

        for _ in 0..calls_per_sec {
            let limiter = limiter.clone();
            let accepted = accepted.clone();
            let rejected = rejected.clone();
            let ip = ip.clone();

            let handle = task::spawn(async move {
                if limiter.check_key(&ip).is_ok() {
                    let mut acc = accepted.lock().unwrap();
                    *acc += 1;
                } else {
                    let mut rej = rejected.lock().unwrap();
                    *rej += 1;
                }
            });

            handles.push(handle);
        }

        for h in handles {
            h.await.unwrap();
        }

        sleep(Duration::from_secs(1)).await;
    }

    let acc = *accepted.lock().unwrap();
    let rej = *rejected.lock().unwrap();
    println!("Run complete (2 minutes):");
    println!("Accepted: {acc}");
    println!("Rejected: {rej}");
}
