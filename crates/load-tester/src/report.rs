use std::time::Duration;

use crate::WorkerStats;

pub struct ScenarioResult {
    pub name: String,
    pub connections: usize,
    pub message_size: usize,
    pub report: Report,
}

pub fn print_matrix(results: &[ScenarioResult]) {
    println!("\n{}", "=".repeat(100));
    println!("{:^100}", "LOAD TEST MATRIX RESULTS");
    println!("{}", "=".repeat(100));
    println!(
        "{:<20} {:>8} {:>8} {:>10} {:>12} {:>10} {:>10} {:>10} {:>6}",
        "Scenario", "Conns", "MsgSize", "Requests", "RPS", "p50", "p95", "p99", "Err%"
    );
    println!("{}", "-".repeat(100));

    for r in results {
        println!(
            "{:<20} {:>8} {:>8} {:>10} {:>12.0} {:>10} {:>10} {:>10} {:>5.1}%",
            truncate(&r.name, 20),
            r.connections,
            format_bytes(r.message_size),
            r.report.total_requests,
            r.report.requests_per_sec,
            format_duration(r.report.latency_p50),
            format_duration(r.report.latency_p95),
            format_duration(r.report.latency_p99),
            r.report.error_rate
        );
    }

    println!("{}", "=".repeat(100));
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{}MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{}B", bytes)
    }
}

pub struct Report {
    pub duration: Duration,
    pub connections: usize,
    pub total_requests: u64,
    pub requests_per_sec: f64,
    pub throughput_up: f64,
    pub throughput_down: f64,
    pub latency_min: Duration,
    pub latency_avg: Duration,
    pub latency_p50: Duration,
    pub latency_p95: Duration,
    pub latency_p99: Duration,
    pub latency_max: Duration,
    pub total_errors: u64,
    pub error_rate: f64,
}

impl Report {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn from_stats(duration: Duration, connections: usize, stats: Vec<WorkerStats>) -> Self {
        let total_requests: u64 = stats.iter().map(|s| s.requests).sum();
        let bytes_sent: u64 = stats.iter().map(|s| s.bytes_sent).sum();
        let bytes_received: u64 = stats.iter().map(|s| s.bytes_received).sum();
        let total_errors: u64 = stats.iter().map(|s| s.errors).sum();

        let duration_secs = duration.as_secs_f64();
        let requests_per_sec = total_requests as f64 / duration_secs;
        let throughput_up = bytes_sent as f64 / duration_secs;
        let throughput_down = bytes_received as f64 / duration_secs;

        let total_ops = total_requests + total_errors;
        let error_rate = if total_ops > 0 {
            total_errors as f64 / total_ops as f64 * 100.0
        } else {
            0.0
        };

        let mut all_latencies: Vec<Duration> =
            stats.into_iter().flat_map(|s| s.latencies).collect();
        all_latencies.sort();

        let (latency_min, latency_avg, latency_p50, latency_p95, latency_p99, latency_max) =
            if all_latencies.is_empty() {
                (
                    Duration::ZERO,
                    Duration::ZERO,
                    Duration::ZERO,
                    Duration::ZERO,
                    Duration::ZERO,
                    Duration::ZERO,
                )
            } else {
                let sum: Duration = all_latencies.iter().sum();
                let avg = sum / all_latencies.len() as u32;
                (
                    all_latencies[0],
                    avg,
                    percentile(&all_latencies, 50.0),
                    percentile(&all_latencies, 95.0),
                    percentile(&all_latencies, 99.0),
                    all_latencies[all_latencies.len() - 1],
                )
            };

        Self {
            duration,
            connections,
            total_requests,
            requests_per_sec,
            throughput_up,
            throughput_down,
            latency_min,
            latency_avg,
            latency_p50,
            latency_p95,
            latency_p99,
            latency_max,
            total_errors,
            error_rate,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn print(&self) {
        println!("=== Load Test Results ===");
        println!("Duration:        {:.1}s", self.duration.as_secs_f64());
        println!("Connections:     {}", self.connections);
        println!("Total requests:  {}", self.total_requests);
        println!("Requests/sec:    {:.1}", self.requests_per_sec);
        println!();
        println!("Throughput:");
        println!(
            "  Upstream:      {}",
            format_bytes_per_sec(self.throughput_up)
        );
        println!(
            "  Downstream:    {}",
            format_bytes_per_sec(self.throughput_down)
        );
        println!();
        println!("Latency:");
        println!("  Min:           {}", format_duration(self.latency_min));
        println!("  Avg:           {}", format_duration(self.latency_avg));
        println!("  p50:           {}", format_duration(self.latency_p50));
        println!("  p95:           {}", format_duration(self.latency_p95));
        println!("  p99:           {}", format_duration(self.latency_p99));
        println!("  Max:           {}", format_duration(self.latency_max));
        println!();
        println!(
            "Errors:          {} ({:.2}%)",
            self.total_errors, self.error_rate
        );
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((sorted.len() as f64 * p / 100.0) as usize).min(sorted.len() - 1);
    sorted[idx]
}

#[allow(clippy::cast_precision_loss)]
fn format_duration(d: Duration) -> String {
    let micros = d.as_micros();
    if micros >= 1_000_000 {
        format!("{:.1}s", d.as_secs_f64())
    } else if micros >= 1000 {
        format!("{:.1}ms", micros as f64 / 1000.0)
    } else {
        format!("{}us", micros)
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_bytes_per_sec(bytes: f64) -> String {
    if bytes >= 1_000_000.0 {
        format!("{:.1} MB/s", bytes / 1_000_000.0)
    } else if bytes >= 1000.0 {
        format!("{:.1} KB/s", bytes / 1000.0)
    } else {
        format!("{:.0} B/s", bytes)
    }
}
