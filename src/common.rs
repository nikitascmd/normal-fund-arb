// pub enum Exchange {
//     Aster,
//     Binance,
//     Hyperliquid,
//     Bybit,
// }

#[derive(Debug)]
pub struct FundingRate {
    pub exchange: String,
    pub asset: String,
    pub funding_rate_pct_one_hour: f64,
    pub funding_rate_pct_two_hours: f64,
    pub funding_rate_pct_four_hours: f64,
    pub funding_rate_pct_eight_hours: f64,
}

pub fn calculate_funding_rate_pcts(rate: &f64, hours: &u64) -> Option<(f64, f64, f64, f64)> {
    let (one_hr, two_hr, four_hr, eight_hr) = match hours {
        1 => (*rate, rate * 2.0, rate * 4.0, rate * 8.0),
        2 => (rate / 2.0, *rate, rate * 2.0, rate * 4.0),
        4 => (rate / 4.0, rate / 2.0, *rate, rate * 2.0),
        8 => (rate / 8.0, rate / 4.0, rate / 2.0, *rate),
        _ => return None,
    };

    Some((
        one_hr * 100.0,
        two_hr * 100.0,
        four_hr * 100.0,
        eight_hr * 100.0,
    ))
}

pub trait FundingRateProvider {
    async fn get_funding_rates(&self) -> Result<Vec<FundingRate>, Box<dyn std::error::Error>>;
}
