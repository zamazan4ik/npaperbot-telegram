use std::str::FromStr;

pub struct Parameters {
    pub bot_name: String,
    pub is_webhook_mode_enabled: bool,
    pub papers_database_uri: url::Url,
    pub max_results_per_request: u8,
    pub database_update_periodicity: chrono::Duration,
}

impl Parameters {
    pub fn new() -> Self {
        let bot_name = std::env::var("BOT_NAME").expect("BOT_NAME env var is not specified");

        let is_webhook_mode_enabled: bool = std::env::var("WEBHOOK_MODE")
            .unwrap_or("false".to_string())
            .parse()
            .expect(
                "Cannot convert WEBHOOK_MODE to bool. Applicable values are only \"true\" or \"false\"",
            );

        let papers_database_uri = url::Url::from_str(
            std::env::var("PAPERS_DATABASE_URI")
                .unwrap_or("https://wg21.link/index.json".to_string())
                .as_str(),
        )
        .expect("Cannot parse PAPERS_DATABASE_URI as URI");

        let max_results_per_request = std::env::var("MAX_RESULTS_PER_REQUEST")
            .unwrap_or("20".to_string())
            .parse::<u8>()
            .expect("Cannot parse MAX_RESULTS_PER_REQUEST as u8");

        let database_update_periodicity = chrono::Duration::hours(
            std::env::var("DATABASE_UPDATE_PERIODICITY_IN_HOURS")
                .unwrap_or("1".to_string())
                .parse::<i64>()
                .expect("Cannot parse DATABASE_UPDATE_PERIODICITY_IN_HOURS as i64"),
        );

        Self {
            bot_name,
            is_webhook_mode_enabled,
            papers_database_uri,
            max_results_per_request,
            database_update_periodicity,
        }
    }
}
