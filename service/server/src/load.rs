use scraper::Html;

use std::io;
use std::fs;
use std::path::PathBuf;

use tracing::info;

#[tracing::instrument]
pub(crate) async fn load_file(filename: &PathBuf) -> io::Result<Html> {
    let document = fs::read_to_string(filename)?;
    let document = Html::parse_document(&document);
    info!("document load from file");
    Ok(document)
}
