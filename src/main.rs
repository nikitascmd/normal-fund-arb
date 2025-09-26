mod aster;
mod common;
mod hyperliquid;
mod telegram_bot;

use std::clone;

use dotenv::dotenv;

use crate::common::{FundingRate, FundingRateProvider};

use tokio::time::{Duration, interval};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let aster_api = aster::AsterApi::new(client.clone());
    let hyperliquid_api = hyperliquid::HyperliquidApi::new(client.clone());
    let telegram_bot = telegram_bot::TelegramBot::new(client.clone());

    let mut interval = interval(Duration::from_secs(60 * 5));

    loop {
        interval.tick().await;

        let aster_rates = aster_api.get_funding_rates().await.unwrap();
        let hyper_rates = hyperliquid_api.get_funding_rates().await.unwrap();

        let mut rates: Vec<FundingRate> = aster_rates.into_iter().chain(hyper_rates).collect();
        rates.sort_by(|a, b| {
            a.funding_rate_pct_eight_hours
                .partial_cmp(&b.funding_rate_pct_eight_hours)
                .unwrap()
        });

        let smallest_rates: Vec<&FundingRate> = rates.iter().take(15).collect();
        let largest_rates: Vec<&FundingRate> = rates.iter().rev().take(15).collect();

        telegram_bot
            .send_message(largest_rates, smallest_rates)
            .await
            .unwrap();
    }
}
