use std::collections::HashMap;
use std::fs;

use askama::Template;
use clap::Parser;
use color_eyre::eyre::Result;
use futures_lite::StreamExt;
use lazy_static::lazy_static;
use scraper::Selector;

use crate::{nix::DataNixTemplate, shasums::ShasumsText};

mod client;
mod nix;
mod shasums;

static NODEJS_DIST_URL: &str = "https://nodejs.org/dist/";

lazy_static! {
    static ref ANCHOR_SELECTOR: Selector = Selector::parse("a").expect("parse anchor selector");
}

#[derive(Parser, Debug)]
struct Args {
    /// Output file path to write the generated Nix data
    #[clap(long, short)]
    output: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = client::Client::new()?;

    let index = client.get_html(NODEJS_DIST_URL).await?;

    let entry_stream = futures_lite::stream::iter(index.select(&ANCHOR_SELECTOR).into_iter());

    let template = entry_stream
        .skip(635) // TODO: Remove this! It's just for testing.
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
            let system_packages = shasums_text
                .entries()
                .map(|entry| {
                    let system = nix::System::from(entry.target);
                    let url = format!("{}{}/{}", NODEJS_DIST_URL, directory, entry.filepath);
                    let package = nix::PackageData::new(&url, entry.checksum);

                    (system, package)
                })
                .collect::<HashMap<_, _>>();

            nix::VersionData {
                directory,
                system_packages,
            }
        })
        .collect::<DataNixTemplate>()
        .await;

    let rendered = template.render()?;

    if let Some(filepath) = args.output {
        fs::write(&filepath, rendered)?;
    } else {
        println!("{rendered}");
    }

    Ok(())
}
