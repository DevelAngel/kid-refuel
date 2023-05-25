use scraper::Html;

use url::Url;
use tracing::info;

pub(crate) async fn download(url: &Url) -> Result<Html, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url.as_str()).await?;
    let document = resp.text().await?;
    let document = Html::parse_document(&document);
    info!("document downloaded");
    Ok(document)
}
