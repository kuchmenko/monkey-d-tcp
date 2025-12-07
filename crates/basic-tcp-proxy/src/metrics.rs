use std::{net::SocketAddr, time::Duration};

use tokio::{
    select,
    sync::{mpsc, watch},
    time::interval,
};

#[derive(Debug, Clone, Copy)]
pub enum MetricEvent {
    ConnectionOpened(SocketAddr),
    ConnectionClosed(SocketAddr),
    BytesUpstream(SocketAddr, u64),
    BytesDownstream(SocketAddr, u64),
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct MetricsSnapshot {
    pub active_connections: u64,
    pub total_connections: u64,
    pub bytes_upstream: u64,
    pub bytes_downstream: u64,
}

impl MetricsSnapshot {
    pub fn to_plain_text(&self) -> String {
        format!(
            "connections_active {}\nconnections_total {}\nbytes_upstream {}\nbytes_downstream {}",
            self.active_connections,
            self.total_connections,
            self.bytes_upstream,
            self.bytes_downstream
        )
    }

    #[allow(clippy::cast_precision_loss)]
    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1_000_000 {
            format!("{:.1}MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.1}KB", bytes as f64 / 1_000.0)
        } else {
            format!("{}B", bytes)
        }
    }

    pub fn log_summary(&self) {
        println!(
            "[METRICS] active: {} | total: {} | up: {} | down: {}",
            self.active_connections,
            self.total_connections,
            Self::format_bytes(self.bytes_upstream),
            Self::format_bytes(self.bytes_downstream)
        );
    }
}

pub struct MetricsCollector {
    rx: mpsc::Receiver<MetricEvent>,
    watch_tx: watch::Sender<MetricsSnapshot>,
    state: MetricsSnapshot,
    log_interval: Duration,
}

impl MetricsCollector {
    pub fn new(
        buffer_size: usize,
        log_interval: Duration,
    ) -> (
        Self,
        mpsc::Sender<MetricEvent>,
        watch::Receiver<MetricsSnapshot>,
    ) {
        let (event_tx, event_rx) = mpsc::channel(buffer_size);
        let (watch_tx, watch_rx) = watch::channel(MetricsSnapshot::default());

        let collector = Self {
            rx: event_rx,
            watch_tx,
            state: MetricsSnapshot::default(),
            log_interval,
        };

        (collector, event_tx, watch_rx)
    }

    pub async fn run(mut self) {
        let mut log_timer = interval(self.log_interval);
        log_timer.tick().await;

        loop {
            select! {
                event = self.rx.recv() => {
                    match event {
                        Some(e) => self.handle_event(e),
                        None => break,
                    }
                }
                _ = log_timer.tick() => {
                    self.state.log_summary();
                }
            }
        }
    }

    fn handle_event(&mut self, event: MetricEvent) {
        match event {
            MetricEvent::ConnectionOpened(addr) => {
                self.state.active_connections += 1;
                self.state.total_connections += 1;
                println!(
                    "[METRICS] ConnectionOpened {} | active: {}, total: {}",
                    addr, self.state.active_connections, self.state.total_connections
                );
            }
            MetricEvent::ConnectionClosed(addr) => {
                self.state.active_connections = self.state.active_connections.saturating_sub(1);
                println!(
                    "[METRICS] ConnectionClosed {} | active: {}",
                    addr, self.state.active_connections
                );
            }
            MetricEvent::BytesUpstream(addr, n) => {
                self.state.bytes_upstream += n;
                println!(
                    "[METRICS] {} ↑ {} | total up: {}",
                    addr,
                    n,
                    MetricsSnapshot::format_bytes(self.state.bytes_upstream)
                );
            }
            MetricEvent::BytesDownstream(addr, n) => {
                self.state.bytes_downstream += n;
                println!(
                    "[METRICS] {} ↓ {} | total down: {}",
                    addr,
                    n,
                    MetricsSnapshot::format_bytes(self.state.bytes_downstream)
                );
            }
        }
        let _ = self.watch_tx.send(self.state.clone());
    }
}
