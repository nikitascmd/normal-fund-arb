use crate::common::{self, FundingRateProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FundingInfo {
    funding_rate: String,
    next_funding_time: u64,
    funding_interval_hours: Option<u64>,
}

type FundingRateResponse = Vec<(String, Vec<(String, Option<FundingInfo>)>)>;

pub struct HyperliquidApi {
    client: reqwest::Client,
}

impl FundingRateProvider for HyperliquidApi {
    async fn get_funding_rates(
        &self,
    ) -> Result<Vec<common::FundingRate>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .post("https://api.hyperliquid.xyz/info")
            .json(&serde_json::json!({"type": "predictedFundings"}))
            .send()
            .await?;

        let text = response.text().await?;
        let fetched_rates = serde_json::from_str::<FundingRateResponse>(&text)?;

        let mut normalized_rates: Vec<common::FundingRate> = Vec::new();
        for rate in &fetched_rates {
            let asset_name = rate.0.clone();

            for info in &rate.1 {
                let exchange = info.0.clone();

                if let Some(FundingInfo {
                    funding_rate,
                    funding_interval_hours,
                    ..
                }) = &info.1
                {
                    match (funding_rate.parse::<f64>(), funding_interval_hours) {
                        (Ok(rate_f64), Some(hours)) => {
                            if let Some((
                                funding_rate_pct_one_hour,
                                funding_rate_pct_two_hours,
                                funding_rate_pct_four_hours,
                                funding_rate_pct_eight_hours,
                            )) = common::calculate_funding_rate_pcts(&rate_f64, hours)
                            {
                                normalized_rates.push(common::FundingRate {
                                    exchange,
                                    asset: asset_name.clone(),
                                    funding_rate_pct_one_hour,
                                    funding_rate_pct_two_hours,
                                    funding_rate_pct_four_hours,
                                    funding_rate_pct_eight_hours,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        println!("{}", normalized_rates.len());

        Ok(normalized_rates)
    }
}

impl HyperliquidApi {
    pub fn new(client: reqwest::Client) -> HyperliquidApi {
        HyperliquidApi { client }
    }
}
