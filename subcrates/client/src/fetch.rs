use std::{pin::pin, time::Duration};

use anyhow::{anyhow, bail, Result};
use futures::future::{select, Either};
use protocol::{config::ElectionConfig, vote::Vote};
use reqwasm::http::Response;

pub async fn election_config(addr: String, timeout: Duration) -> Result<ElectionConfig> {
    let addr = format!("{addr}/config");
    let response = get(&addr, timeout).await?;
    if response.status() != 200 {
        bail!("Error code {}", response.status());
    }
    let config: ElectionConfig = response.json().await?;

    Ok(config)
}

pub async fn submit_vote(addr: String, timeout: Duration, vote: Vote) -> Result<()> {
    let addr = format!("{addr}/vote");
    let vote = serde_json::to_string(&vote)?;

    let response = post(vote, &addr, timeout).await?;
    if response.status() != 200 {
        bail!("Error code {}", response.status());
    }

    Ok(())
}

async fn get(addr: &str, timeout: Duration) -> Result<Response> {
    let fetch_future = pin!(async {
        match reqwasm::http::Request::get(&addr).send().await {
            Ok(response) => Ok(response),
            Err(_) => Err(anyhow!("Request failed".to_string())),
        }
    });

    let timeout_future = gloo_timers::future::TimeoutFuture::new(timeout.as_millis().try_into()?);

    match select(fetch_future, timeout_future).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err(anyhow!("Request timed out".to_string())),
    }
}

async fn post(payload: String, addr: &str, timeout: Duration) -> Result<Response> {
    let fetch_future = pin!(async {
        match reqwasm::http::Request::post(&addr)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(_) => Err(anyhow!("Request failed".to_string())),
        }
    });

    let timeout_future = gloo_timers::future::TimeoutFuture::new(timeout.as_millis().try_into()?);

    match select(fetch_future, timeout_future).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err(anyhow!("Request timed out".to_string())),
    }
}
