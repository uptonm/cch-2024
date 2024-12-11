use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;

use leaky_bucket::RateLimiter;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimit(pub Arc<Mutex<RateLimiter>>);

impl Default for RateLimit {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(filled_bucket())))
    }
}

pub fn filled_bucket() -> RateLimiter {
    RateLimiter::builder()
        .max(5)
        .refill(1)
        .interval(Duration::from_secs(1))
        .initial(5)
        .build()
}

impl Deref for RateLimit {
    type Target = Arc<Mutex<RateLimiter>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RateLimit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
