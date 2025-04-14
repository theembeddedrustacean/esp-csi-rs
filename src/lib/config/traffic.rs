/// Traffic Generation Configuration Struct
#[derive(Debug, Clone)]
pub struct TrafficConfig {
    /// Traffic type
    pub traffic_type: TrafficType,
    /// Interval between packets in milliseconds
    pub traffic_interval_ms: u64,
}

/// Options of Traffic Options
#[derive(Debug, Clone)]
pub enum TrafficType {
    /// UDP traffic
    UDP,
    /// ICMP ping
    ICMPPing,
}

impl Default for TrafficConfig {
    /// Default implementation for Traffic Configuration:
    /// - Traffic generation disabled
    /// - Traffic type is ICMP Ping
    /// - Traffic generation interval is 1000 ms
    fn default() -> Self {
        Self {
            traffic_type: TrafficType::ICMPPing,
            traffic_interval_ms: 1000,
        }
    }
}
