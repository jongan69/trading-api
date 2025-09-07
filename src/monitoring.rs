use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub endpoint_stats: HashMap<String, EndpointStats>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub requests: u64,
    pub avg_response_time: f64,
    pub success_rate: f64,
    pub last_error: Option<String>,
}

pub struct MetricsCollector {
    start_time: Instant,
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    response_times: Arc<RwLock<Vec<f64>>>,
    endpoint_metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
}

#[derive(Debug)]
struct EndpointMetrics {
    requests: AtomicU64,
    response_times: Vec<f64>,
    errors: u64,
    last_error: Option<String>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            response_times: Arc::new(RwLock::new(Vec::new())),
            endpoint_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_request(&self, endpoint: &str, response_time: Duration, success: bool, error: Option<String>) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        let response_time_ms = response_time.as_millis() as f64;
        
        // Update global response times
        {
            let mut times = self.response_times.write().await;
            times.push(response_time_ms);
            // Keep only last 1000 response times to prevent memory growth
            if times.len() > 1000 {
                times.remove(0);
            }
        }

        // Update endpoint-specific metrics
        {
            let mut endpoint_metrics = self.endpoint_metrics.write().await;
            let metrics = endpoint_metrics.entry(endpoint.to_string()).or_insert_with(|| EndpointMetrics {
                requests: AtomicU64::new(0),
                response_times: Vec::new(),
                errors: 0,
                last_error: None,
            });

            metrics.requests.fetch_add(1, Ordering::Relaxed);
            metrics.response_times.push(response_time_ms);
            
            // Keep only last 100 response times per endpoint
            if metrics.response_times.len() > 100 {
                metrics.response_times.remove(0);
            }

            if !success {
                metrics.errors += 1;
                metrics.last_error = error;
            }
        }
    }

    pub async fn get_metrics(&self) -> ApiMetrics {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);

        let avg_response_time = {
            let times = self.response_times.read().await;
            if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<f64>() / times.len() as f64
            }
        };

        let endpoint_stats = {
            let metrics = self.endpoint_metrics.read().await;
            let mut stats = HashMap::new();
            
            for (endpoint, metric) in metrics.iter() {
                let requests = metric.requests.load(Ordering::Relaxed);
                let avg_time = if metric.response_times.is_empty() {
                    0.0
                } else {
                    metric.response_times.iter().sum::<f64>() / metric.response_times.len() as f64
                };
                let success_rate = if requests == 0 {
                    0.0
                } else {
                    ((requests - metric.errors) as f64 / requests as f64) * 100.0
                };

                stats.insert(endpoint.clone(), EndpointStats {
                    requests,
                    avg_response_time: avg_time,
                    success_rate,
                    last_error: metric.last_error.clone(),
                });
            }
            
            stats
        };

        ApiMetrics {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            average_response_time: avg_response_time,
            endpoint_stats,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.get_metrics().await;
        
        let success_rate = if metrics.total_requests == 0 {
            100.0
        } else {
            (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0
        };

        let status = if success_rate >= 95.0 && metrics.average_response_time < 5000.0 {
            "healthy"
        } else if success_rate >= 80.0 && metrics.average_response_time < 10000.0 {
            "degraded"
        } else {
            "unhealthy"
        };

        HealthStatus {
            status: status.to_string(),
            success_rate,
            average_response_time: metrics.average_response_time,
            total_requests: metrics.total_requests,
            uptime_seconds: metrics.uptime_seconds,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub success_rate: f64,
    pub average_response_time: f64,
    pub total_requests: u64,
    pub uptime_seconds: u64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
