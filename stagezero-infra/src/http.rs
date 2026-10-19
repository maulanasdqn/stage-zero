use std::time::Duration;

use rand::seq::SliceRandom;
use reqwest::Client;

const USER_AGENTS: [&str; 5] = [
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0",
];

pub fn random_user_agent() -> &'static str {
    USER_AGENTS
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or(USER_AGENTS[0])
}

pub fn build_http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
}
