use scraper::Html;

use std::io::{self, Write};
use std::fs::File;
use std::path::PathBuf;

use tracing::info;

#[tracing::instrument(skip(document))]
pub(crate) async fn save_stdout(document: &Html) -> io::Result<()> {
    let html = document.html();
    let mut out = io::stdout().lock();
    out.write_all(html.as_bytes())?;
    info!("document saved to stdout");
    Ok(())
}

#[tracing::instrument(skip(document))]
pub(crate) async fn save_file(document: &Html, filename: &PathBuf) -> io::Result<()> {
    let html = document.html();
    let mut out = File::create(filename)?;
    out.write_all(html.as_bytes())?;
    out.sync_all()?;
    info!("document saved to file");
    Ok(())
}
