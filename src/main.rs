use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use governor::state::InMemoryState;
use governor::clock::DefaultClock;

// Configuration structure (Hydra-inspired)
struct HydraConfig {
    target_ip: String,
    username_list: Vec<String>,
    password_list: Vec<String>,
    threads: usize,
    timeout_ms: u64,
    max_attempts: Option<usize>,
}

// Hydra-style statistical tracking
struct HydraStats {
    attempts: AtomicUsize,
    successes: AtomicUsize,
    start_time: Instant,
}

impl HydraStats {
    fn new() -> Self {
        HydraStats {
            attempts: AtomicUsize::new(0),
            successes: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    fn print_progress(&self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs();
        let elapsed_nanos = elapsed.as_nanos();
        let attempts = self.attempts.load(Ordering::Relaxed);
        let rate = if elapsed_secs > 0 {
            attempts as f64 / elapsed_secs as f64
        } else {
            0.0
        };

        println!(
            "Progress: {} attempts ({} success) | {:.2} attempts/sec | Elapsed: {}s ({} ns)",
            attempts,
            self.successes.load(Ordering::Relaxed),
            rate,
            elapsed_secs,
            elapsed_nanos
        );
    }
}

fn main() -> io::Result<()> {
    // Hydra-like banner
    println!(r#"
     _  _   _ _____ _   _     
    | || | | |  _  | \ | |    
    | || |_| | | | |  \| |    
    |__   _| | | | | . ` |    
       | | \ \_/ / | |\  |    
       |_|  \___/|_|_| \_|    
    Security Testing Tool (Educational)
    "#);

    // Get input
    let ip = get_input("Target IP (or file path): ");
    let users_file = get_input("Username list path: ");
    let pass_file = get_input("Password list path: ");
    let threads: usize = get_input("Threads (1-10): ").parse().unwrap_or(4);
    let timeout: u64 = get_input("Timeout per attempt (ms): ").parse().unwrap_or(1000);

    // Load wordlists
    let (ips, users, passwords) = if Path::new(&ip).exists() {
        (read_lines(&ip)?, read_lines(&users_file)?, read_lines(&pass_file)?)
    } else {
        (vec![ip], read_lines(&users_file)?, read_lines(&pass_file)?)
    };

    let config = HydraConfig {
        target_ip: ips[0].clone(), // Simplified for example
        username_list: users,
        password_list: passwords,
        threads,
        timeout_ms: timeout,
        max_attempts: None,
    };

    let stats = Arc::new(HydraStats::new());

    // Hydra-style progress output
    let stats_clone = stats.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        stats_clone.print_progress();
    });

    // Start testing (simulated)
    let rt = Runtime::new().unwrap();
    rt.block_on(run_attack(config, stats));

    Ok(())
}

async fn run_attack(config: HydraConfig, stats: Arc<HydraStats>) {
    let limiter = RateLimiter::<(), InMemoryState, DefaultClock>::direct(
        Quota::per_second(NonZeroU32::new(10).unwrap()), // 10 attempts/sec
    );

    for user in config.username_list {
        for pass in &config.password_list {
            limiter.until_ready().await;

            stats.attempts.fetch_add(1, Ordering::Relaxed);

            // Simulated success condition
            if pass == "admin123" {
                stats.successes.fetch_add(1, Ordering::Relaxed);
                println!("[SUCCESS] {}:{}@{}", user, pass, config.target_ip);
            }

            tokio::time::sleep(Duration::from_millis(config.timeout_ms)).await;
        }
    }
}

// Helper functions

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .filter(|s| !s.is_empty())
        .collect())
}
