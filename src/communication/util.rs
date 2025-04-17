//! Utility functions for communication
use anyhow::Result;
use tokio::time::{sleep, Duration};
use tracing::warn;

/// Retry a future with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    f: F, 
    max_retries: usize, 
    initial_delay: Duration,
) -> Result<T> 
where 
    F: Fn()->Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = initial_delay;
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(e);
                }

                warn!(
                    "Operation failed! (attempt {}/{}), retrying in {:?}: {}",
                    attempt,
                    max_retries,
                    delay,
                    e
                );

                sleep(delay).await;
                delay = delay.saturating_mul(2);
            }
        }
    }
}

/// Calculate exponential backff delay with jitter
pub fn backoff_with_jitter(attempt: usize, base_delay_ms: u64, max_delay_ms: u64) -> Duration {
    use rand::{thread_rng, Rng};
    
    let exp_backoff = base_delay_ms * (1u64 << attempt.min(31));
    let capped_backoff = exp_backoff.min(max_delay_ms);
    
    // Add jitter - random value between 0-20% of the delay
    let jitter_factor = thread_rng().gen_range(0.0..0.2);
    let jitter = (capped_backoff as f64 * jitter_factor) as u64;
    
    Duration::from_millis(capped_backoff + jitter)
}
