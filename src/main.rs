use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FundingRate {
    symbol: String,
    #[serde(rename = "markPrice")]
    mark_price: String,
    #[serde(rename = "indexPrice")]
    index_price: String,
    #[serde(rename = "lastFundingRate")]
    last_funding_rate: String,
    #[serde(rename = "nextFundingTime")]
    next_funding_time: i64,
    time: i64,
}

#[tokio::main]
async fn main() {
    println!("Getting funding rates from Aster...");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let response = client
        .get("https://fapi.asterdex.com/fapi/v1/premiumIndex")
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("Status: {}", resp.status());

            // Get response text first to see what we actually got
            let text = resp.text().await.unwrap();
            println!("Raw response: {}", text);

            // Try to parse as array of funding rates
            match serde_json::from_str::<Vec<FundingRate>>(&text) {
                Ok(mut funding_rates) => {
                    println!("\n📊 Found {} funding rates:", funding_rates.len());

                    // Sort by funding rate (highest first)
                    funding_rates.sort_by(|a, b| {
                        let rate_a = a.last_funding_rate.parse::<f64>().unwrap_or(0.0);
                        let rate_b = b.last_funding_rate.parse::<f64>().unwrap_or(0.0);
                        rate_b
                            .partial_cmp(&rate_a)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    println!("\n🔥 Top funding rates (best arbitrage opportunities):");
                    for rate in funding_rates.iter().take(10) {
                        let funding_pct =
                            rate.last_funding_rate.parse::<f64>().unwrap_or(0.0) * 100.0;

                        println!("  {} - Funding: {:.4}%", rate.symbol, funding_pct);
                    }

                    println!("\n❄️  Lowest funding rates (shorts get paid):");
                    for rate in funding_rates.iter().rev().take(5) {
                        let funding_pct =
                            rate.last_funding_rate.parse::<f64>().unwrap_or(0.0) * 100.0;

                        println!("  {} - Funding: {:.4}%", rate.symbol, funding_pct);
                    }
                }
                Err(e) => {
                    println!("JSON parse error: {}", e);

                    // Maybe it's a single object instead of array?
                    match serde_json::from_str::<FundingRate>(&text) {
                        Ok(single_rate) => {
                            println!("Got single funding rate for: {}", single_rate.symbol);
                        }
                        Err(e2) => println!("Also failed as single object: {}", e2),
                    }
                }
            }
        }
        Err(e) => {
            println!("Connection error: {}", e);
            println!("Error details: {:?}", e);

            // Try a simple ping first
            println!("\nTrying ping endpoint instead...");
            let ping_response = client
                .get("https://fapi.asterdex.com/fapi/v1/ping")
                .send()
                .await;
            match ping_response {
                Ok(resp) => println!("Ping successful! Status: {}", resp.status()),
                Err(e) => println!("Ping also failed: {}", e),
            }
        }
    }
}
