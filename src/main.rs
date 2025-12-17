use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

use askama::Template;
use clap::Parser;
use color_eyre::eyre::{Context, Result};
use futures_lite::StreamExt;
use lazy_static::lazy_static;
use scraper::Selector;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::{nix::DataNixTemplate, shasums::ShasumsText};

mod client;
mod nix;
mod shasums;

static NODEJS_DIST_URL: &str = "https://nodejs.org/dist/";

lazy_static! {
    static ref ANCHOR_SELECTOR: Selector = Selector::parse("a").expect("parse anchor selector");
}

/// This program scrapes the nodejs binary distribution index
/// (https://nodejs.org/dist/) and outputs a Nix attribute set that contains
/// every nodejs version with the URL and sha256 of the system binaries
/// available for each.
///
/// By default, this will output to stdout, but you can instead write to a file
/// via the `--output` flag.
#[derive(Parser, Debug)]
struct Args {
    /// Output file path to write the generated Nix data
    #[clap(long, short)]
    output: Option<std::path::PathBuf>,

    /// The number of node version listings to skip over. Use in conjunction
    /// with `--take` to simulate pagination.
    #[clap(long, default_value_t = 0)]
    skip: usize,

    /// The maximum number of node version listings to process. Can be used in
    /// conjunction with `--skip` to simulate pagination.
    #[clap(long, default_value_t = std::usize::MAX)]
    take: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    configure_telemetry()?;
    let args = Args::parse();

    let client = client::Client::new()?;

    let index = client.get_html(NODEJS_DIST_URL).await?;

    // The distribution index is a typical HTML page with links for every file
    // and subdirectory, so we want to iterator over the anchor tags.
    let entry_stream = futures_lite::stream::iter(index.select(&ANCHOR_SELECTOR).into_iter());

    tracing::info!("Successfully fetched {NODEJS_DIST_URL}");

    let template = entry_stream
        .filter_map(|anchor| {
            // The anchor tags will link to the version directories.
            // E.g. `v0.10.39/` or `latest-v8.x/`
            let directory = anchor.inner_html();

            // There maybe some entries that aren't version directories.
            // We only want the version directories.
            let (directory, _) = directory.rsplit_once('/')?;

            // Also skip the `..` directory since it's not a version.
            if directory == ".." {
                return None;
            }

            Some(directory.to_string())
        })
        .then(|directory| async {
            // Every version directory should contain a `SHASUMS256.txt` file
            // which lists the files in that directory and their corresponding
            // checksums. This is where all the information we need for the
            // ouput is.
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
        .skip(args.skip)
        .take(args.take)
        .inspect(|version| {
            tracing::info!(entry = version.directory, "Succesfully processed");
        })
        .collect::<DataNixTemplate>()
        .await;

    // We either want to write to stdout or a file if one is given, both of
    // which implement [`io::Write`], so we can create a single writer box for
    // the template to write into.
    let mut writer: Box<dyn io::Write> = if let Some(filepath) = args.output {
        tracing::info!("Writing to {}", filepath.display());
        Box::new(File::create(&filepath)?)
    } else {
        tracing::info!("Writing to stdout");
        Box::new(io::stdout())
    };

    // Askama will render the template chunk by chunk into our output box.
    template.write_into(&mut writer)?;

    // Insert a final newline for convention.
    writer.write(b"\n")?;

    Ok(())
}

fn configure_telemetry() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .compact()
                .without_time()
                .with_file(false)
                .with_line_number(false)
                .with_target(false),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(
                    "info"
                        .parse()
                        .expect("default filter directive should be valid"),
                )
                .from_env_lossy(),
        )
        .try_init()
        .context("configure local tracing subscriber")
}
