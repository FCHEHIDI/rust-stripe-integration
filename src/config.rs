use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            stripe_secret_key: env::var("STRIPE_SECRET_KEY")
                .expect("STRIPE_SECRET_KEY doit être défini dans .env"),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")
                .unwrap_or_else(|_| String::from("")),
            base_url: env::var("BASE_URL")
                .unwrap_or_else(|_| String::from("http://localhost:3000")),
        })
    }
}
