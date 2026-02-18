use http::HeaderValue;
use log::warn;
use reqwest::{Request, Response, StatusCode};
use reqwest_middleware::{Middleware, Next, Result};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
struct RateState {
    remaining: Option<u64>,
    reset_after_secs: Option<u64>, // seconds until reset
}

#[derive(Clone, Debug)]
pub struct RobloxRateLimitMiddleware {
    max_429_retries: usize,
    cushion_ms: u64,
}

#[derive(Clone, Debug)]
pub struct RobloxAuthMiddleware {
    api_token: Arc<Mutex<Option<String>>>,
}

impl RobloxRateLimitMiddleware {
    pub fn new() -> Self {
        Self {
            max_429_retries: 5,
            cushion_ms: 75,
        }
    }

    pub fn with_max_429_retries(mut self, n: usize) -> Self {
        self.max_429_retries = n;
        self
    }

    fn retry_wait_from_headers(resp: &Response) -> Duration {
        let secs = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .or_else(|| {
                resp.headers()
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.trim().parse::<u64>().ok())
            })
            .unwrap_or(1);

        Duration::from_secs(secs)
    }
}

impl RobloxAuthMiddleware {
    pub fn new() -> Self {
        Self {
            api_token: super::API_TOKEN.clone(),
        }
    }

    pub async fn get_api_token(&self) -> Option<String> {
        let token_lock = self.api_token.lock().await;
        token_lock.clone()
    }
}

#[async_trait::async_trait]
impl Middleware for RobloxAuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut http::Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        if let Some(token) = self.get_api_token().await {
            req.headers_mut()
                .insert("x-api-key", HeaderValue::from_str(&token).unwrap());
        }

        let resp = next.run(req, extensions).await?;
        Ok(resp)
    }
}

#[async_trait::async_trait]
impl Middleware for RobloxRateLimitMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut http::Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let mut req = req;
        for attempt in 0..=self.max_429_retries {
            let req_clone = req.try_clone();
            let resp = next.clone().run(req, extensions).await?;

            if resp.status() != StatusCode::TOO_MANY_REQUESTS {
                return Ok(resp);
            }

            if attempt >= self.max_429_retries {
                return Ok(resp);
            }

            let wait = Self::retry_wait_from_headers(&resp);

            warn!(
                "Rate limited on attempt {}, retrying after {} seconds...",
                attempt + 1,
                wait.as_secs()
            );

            tokio::time::sleep(wait + Duration::from_millis(self.cushion_ms)).await;

            if let Some(cloned) = req_clone {
                req = cloned;
            } else {
                return Ok(resp);
            }
        }

        unreachable!()
    }
}
