use color_eyre::eyre::{Context, Result};
use scraper::Html;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static REQUEST_TIMEOUT_SECS: u64 = 10;

pub struct Client {
    inner: reqwest::Client,
}

impl From<reqwest::Client> for Client {
    fn from(value: reqwest::Client) -> Self {
        Self { inner: value }
    }
}

impl Client {
    pub fn new() -> Result<Self> {
        reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .context("build client")
            .map(Self::from)
    }

    pub async fn get_html(&self, url: &str) -> Result<Html> {
        self.get_text(url)
            .await
            .map(|text| Html::parse_document(&text))
    }

    pub async fn get_text(&self, url: &str) -> Result<String> {
        self.inner
            .get(url)
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .context("get response")?
            .text()
            .await
            .context("get text")
    }
}
