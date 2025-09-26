use crate::common;
use serde_json::json;
use std::env;

pub struct TelegramBot {
    bot_token: String,
    client: reqwest::Client,
    chat_id: String,
}

impl TelegramBot {
    pub fn new(client: reqwest::Client) -> TelegramBot {
        let bot_token =
            env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set in .env");
        let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set in .env");
        TelegramBot {
            bot_token,
            client,
            chat_id,
        }
    }

    fn format_rates(&self, rates: &[&common::FundingRate], title: &str) -> String {
        let mut message = format!("🔸 *{}*\n\n", title);

        for rate in rates {
            message.push_str(&format!(
                "📈 *{}* on *{}*\n\
                ├ 1h: {:.4}%\n\
                ├ 2h: {:.4}%\n\
                ├ 4h: {:.4}%\n\
                └ 8h: {:.4}%\n\n",
                rate.asset,
                rate.exchange,
                rate.funding_rate_pct_one_hour,
                rate.funding_rate_pct_two_hours,
                rate.funding_rate_pct_four_hours,
                rate.funding_rate_pct_eight_hours
            ));
        }

        message
    }

    pub async fn send_message(
        &self,
        largest_rates: Vec<&common::FundingRate>,
        smallest_rates: Vec<&common::FundingRate>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let mut message = String::new();

        message.push_str("🚀 *Funding Rate Report*\n");
        message.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

        if !largest_rates.is_empty() {
            message.push_str(&self.format_rates(&largest_rates, "Highest Funding Rates 📊"));
        }

        if !largest_rates.is_empty() && !smallest_rates.is_empty() {
            message.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
        }

        if !smallest_rates.is_empty() {
            message.push_str(&self.format_rates(&smallest_rates, "Lowest Funding Rates 📉"));
        }

        message.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        message.push_str(&format!(
            "📅 Generated at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));

        let payload = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "Markdown"
        });

        let response = self.client.post(url).json(&payload).send().await?;

        if response.status().is_success() {
            println!("Message sent successfully!");
        } else {
            let error_text = response.text().await?;
            println!("Failed to send message: {}", error_text);
        }

        Ok(())
    }
}
