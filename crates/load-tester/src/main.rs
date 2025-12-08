use std::time::{Duration, Instant};

use load_tester::{Config, Report, ScenarioResult, print_matrix, run_worker};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("load_test.toml").unwrap_or_else(|e| {
        eprintln!("Failed to load config: {e}, using defaults");
        Config::default()
    });

    if config.is_matrix_mode() {
        run_matrix_mode(&config).await;
    } else {
        run_single_mode(&config).await;
    }

    Ok(())
}

async fn run_single_mode(config: &Config) {
    println!("========================================");
    println!("        Load Tester Starting");
    println!("========================================");
    println!("Target:          {}", config.target_addr);
    println!("Connections:     {}", config.connections);
    println!("Duration:        {}s", config.duration_secs);
    println!("Message size:    {} bytes", config.message_size);
    println!("----------------------------------------");
    println!("Press Ctrl+C to stop early");
    println!("========================================\n");

    let report = run_scenario(
        &config.target_addr,
        config.connections,
        config.duration_secs,
        config.message_size,
    )
    .await;

    println!();
    report.print();
}

async fn run_matrix_mode(config: &Config) {
    println!("========================================");
    println!("     Load Tester - Matrix Mode");
    println!("========================================");
    println!("Target:          {}", config.target_addr);
    println!("Scenarios:       {}", config.scenarios.len());
    println!("----------------------------------------");
    println!("Press Ctrl+C to abort");
    println!("========================================\n");

    let mut results = Vec::with_capacity(config.scenarios.len());

    for (i, scenario) in config.scenarios.iter().enumerate() {
        println!(
            "[{}/{}] Running: {} (conns={}, msg={}B, dur={}s)",
            i + 1,
            config.scenarios.len(),
            scenario.name,
            scenario.connections,
            scenario.message_size,
            scenario.duration_secs
        );

        let report = run_scenario(
            &config.target_addr,
            scenario.connections,
            scenario.duration_secs,
            scenario.message_size,
        )
        .await;

        println!(
            "         -> {:.0} RPS, p99={:.0}us, err={:.1}%\n",
            report.requests_per_sec,
            report.latency_p99.as_micros(),
            report.error_rate
        );

        results.push(ScenarioResult {
            name: scenario.name.clone(),
            connections: scenario.connections,
            message_size: scenario.message_size,
            report,
        });
    }

    print_matrix(&results);
}

async fn run_scenario(
    target_addr: &str,
    connections: usize,
    duration_secs: u64,
    message_size: usize,
) -> Report {
    let shutdown = CancellationToken::new();
    let start = Instant::now();

    let mut handles = Vec::with_capacity(connections);
    for _ in 0..connections {
        let target = target_addr.to_string();
        let token = shutdown.clone();
        handles.push(tokio::spawn(async move {
            run_worker(target, message_size, token).await
        }));
    }

    let duration = Duration::from_secs(duration_secs);
    select! {
        _ = sleep(duration) => {}
        _ = tokio::signal::ctrl_c() => {
            println!("\n[LOAD-TESTER] Ctrl+C received, aborting...");
            std::process::exit(0);
        }
    }

    shutdown.cancel();

    let mut all_stats = Vec::with_capacity(connections);
    for handle in handles {
        if let Ok(stats) = handle.await {
            all_stats.push(stats);
        }
    }

    let actual_duration = start.elapsed();
    Report::from_stats(actual_duration, connections, all_stats)
}
