use std::sync::atomic::AtomicU64;

#[derive(Debug, Default)]
pub struct Metrics {
    pub active_connections: AtomicU64,
    pub total_connections: AtomicU64,
    pub bytes_upstream: AtomicU64,
    pub bytes_downstream: AtomicU64,
}
