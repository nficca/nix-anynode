use clap::Parser;
use color_eyre::eyre::{Context, Result};
use futures_lite::StreamExt;
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use semver::Version;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static NODEJS_DIST_URL: &str = "https://nodejs.org/dist/";
static REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref ANCHOR_SELECTOR: Selector = Selector::parse("a").expect("parse anchor selector");
}

#[derive(Parser, Debug)]
struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = Client::new()?;

    let index = client.get_html(NODEJS_DIST_URL).await?;

    let entry_stream = futures_lite::stream::iter(index.select(&ANCHOR_SELECTOR).into_iter());

    entry_stream
        .filter_map(|anchor| {
            // The anchor tags will link to the version directories.
            // E.g. `v0.10.39/` or `latest-v8.x/`
            let directory = anchor.inner_html();

            // Strip the `v` prefix
            let stripped = match directory.split_once('v')? {
                ("", stripped) => stripped,
                _ => return None,
            };

            // Strip the `/` suffix
            let version = match stripped.rsplit_once('/')? {
                (version, "") => Version::parse(version).ok()?,
                _ => return None,
            };

            Some(IndexEntry { directory, version })
        })
        .then(|entry| async {
            let url = format!("{}/{}", NODEJS_DIST_URL, entry.directory);
            match client.get_html(&url).await {
                Ok(html) => Some((entry, html)),
                _ => None,
            }
        })
        .filter_map(|option| option)
        .for_each(|(entry, html)| {
            println!("{}", entry.version);
            for element in html.select(&ANCHOR_SELECTOR) {
                let inner_html = element.inner_html();
                println!("  {}", inner_html);
            }
        })
        .await;

    Ok(())
}

struct Client {
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
        self.inner
            .get(url)
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .context("get html request")?
            .text()
            .await
            .context("get html request text")
            .map(|text| Html::parse_document(&text))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct IndexEntry {
    version: Version,
    directory: String,
}
