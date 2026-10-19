use std::time::Duration;

use rand::Rng;
use stagezero_core::traits::Delay;

#[derive(Debug, Clone, Copy)]
pub struct RandomDelay {
    min: Duration,
    max: Duration,
}

impl RandomDelay {
    pub fn new(min: Duration, max: Duration) -> Self {
        Self {
            min: min.min(max),
            max: max.max(min),
        }
    }

    fn pick(&self) -> Duration {
        let min_millis = self.min.as_millis() as u64;
        let max_millis = self.max.as_millis() as u64;
        Duration::from_millis(rand::thread_rng().gen_range(min_millis..=max_millis))
    }
}

impl Delay for RandomDelay {
    async fn wait(&self) {
        let duration = self.pick();
        tokio::time::sleep(duration).await;
    }
}
