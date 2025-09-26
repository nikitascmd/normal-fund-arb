use std::{collections::HashMap, env, f64};

use reqwest;
use serde::Deserialize;

use crate::common::{self, FundingRateProvider};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AsterFundingRate {
    symbol: String,
    mark_price: String,
    index_price: String,
    last_funding_rate: String,
    next_funding_time: i64,
    time: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AsterFundingInfo {
    symbol: String,
    funding_interval_hours: u64,
}

#[derive(Debug)]
pub struct AsterApi {
    api_key: String,
    api_secret: String,
    client: reqwest::Client,
}

impl FundingRateProvider for AsterApi {
    async fn get_funding_rates(
        &self,
    ) -> Result<Vec<common::FundingRate>, Box<dyn std::error::Error>> {
        let funding_rates_response: reqwest::Response = self
            .client
            .get("https://fapi.asterdex.com/fapi/v1/premiumIndex")
            .send()
            .await?;

        let funding_rates_text = funding_rates_response.text().await?;
        let fetched_rates = serde_json::from_str::<Vec<AsterFundingRate>>(&funding_rates_text)?;

        let funding_info_response = self
            .client
            .get("https://fapi.asterdex.com/fapi/v1/fundingInfo")
            .send()
            .await?;
        let funding_info_text = funding_info_response.text().await?;
        let funding_infos = serde_json::from_str::<Vec<AsterFundingInfo>>(&funding_info_text)?;

        let symbol_to_funding_info: HashMap<String, AsterFundingInfo> = funding_infos
            .into_iter()
            .map(|info| (info.symbol.clone(), info))
            .collect();

        let funding_rates: Vec<common::FundingRate> = fetched_rates
            .into_iter()
            .filter_map(|rate| {
                let funding_info = symbol_to_funding_info.get(&rate.symbol)?;

                if let Ok(rate_f64) = rate.last_funding_rate.parse::<f64>() {
                    if let Some((
                        funding_rate_pct_one_hour,
                        funding_rate_pct_two_hours,
                        funding_rate_pct_four_hours,
                        funding_rate_pct_eight_hours,
                    )) = common::calculate_funding_rate_pcts(
                        &rate_f64,
                        &funding_info.funding_interval_hours,
                    ) {
                        return Some(common::FundingRate {
                            exchange: "AsterPerp".to_string(),
                            asset: rate.symbol,
                            funding_rate_pct_one_hour,
                            funding_rate_pct_two_hours,
                            funding_rate_pct_four_hours,
                            funding_rate_pct_eight_hours,
                        });
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            })
            .collect();

        Ok(funding_rates)
    }
}

impl AsterApi {
    pub fn new(client: reqwest::Client) -> AsterApi {
        let api_key =
            env::var("ASTER_API_KEY").expect("ASTER_API_KEY environment variable must be set");
        let api_secret = env::var("ASTER_API_SECRET")
            .expect("ASTER_API_SECRET environment variable must be set");

        AsterApi {
            api_key,
            api_secret,
            client,
        }
    }
}
