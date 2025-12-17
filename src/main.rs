use clap::Parser;
use color_eyre::eyre::{Context, Result};
use futures_lite::StreamExt;
use lazy_static::lazy_static;
use scraper::{Html, Selector};

use crate::shasums::ShasumsText;

mod shasums;

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

    let s = entry_stream
        .skip(800) // TODO: Remove this! It's just for testing.
        .take(5) // TODO: Remove this! It's just for testing.
        .filter_map(|anchor| {
            // The anchor tags will link to the version directories.
            // E.g. `v0.10.39/` or `latest-v8.x/`
            let directory = anchor.inner_html();

            // There maybe some entries that aren't version directories.
            // We only want the version directories.
            let (directory, _) = directory.rsplit_once('/')?;

            Some(directory.to_string())
        })
        .then(|directory| async {
            let url = format!("{}/{}/SHASUMS256.txt", NODEJS_DIST_URL, directory);
            match client.get_text(&url).await {
                Ok(shasums) => Some((directory, ShasumsText::from(shasums))),
                _ => None,
            }
        })
        .filter_map(|option| option)
        .map(|(directory, shasums_text)| {
            let mut version_entry = format!("\"{directory}\" = {{\n");
            for entry in shasums_text.entries() {
                let mut target_entry = format!("\"{}\" = {{\n", entry.target);
                target_entry.push_str(&format!(
                    "url = \"{}{}/{}\";\n",
                    NODEJS_DIST_URL, directory, entry.filepath
                ));
                target_entry.push_str(&format!("sha256 = \"{}\";\n", entry.checksum));
                target_entry.push_str("};\n");

                version_entry.push_str(&target_entry);
            }
            version_entry.push_str("};\n");
            version_entry
        })
        .collect::<String>()
        .await;

    println!("{{ {s} }}");

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
