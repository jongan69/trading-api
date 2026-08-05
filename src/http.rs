use std::sync::OnceLock;
use std::time::Duration;
use reqwest::Client;

static SHARED_CLIENT: OnceLock<Client> = OnceLock::new();

/// Returns a shared `reqwest::Client` with connection pooling, timeouts,
/// and keepalive configured. Every HTTP call in the crate should use this
/// client (or a builder cloned from it) instead of `Client::new()`.
pub fn shared_client() -> &'static Client {
    SHARED_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(20)
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent(concat!("trading-api/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create shared HTTP client")
    })
}
