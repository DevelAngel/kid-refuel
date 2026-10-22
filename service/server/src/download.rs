use scraper::Html;

use std::time::Instant;
use tracing::info;
use url::Url;

pub(crate) struct DownloadedPage {
    pub document: Html,
    pub body: String,
    pub status: http::StatusCode,
    pub duration: std::time::Duration,
}

pub(crate) async fn download_with_details(
    url: &Url,
) -> Result<DownloadedPage, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let response = reqwest::get(url.as_str()).await?;
    let status = http::StatusCode::from_u16(response.status().as_u16())?;
    let body = response.text().await?;
    let duration = started.elapsed();
    let document = Html::parse_document(&body);
    info!("document downloaded");
    Ok(DownloadedPage {
        document,
        body,
        status,
        duration,
    })
}

pub(crate) async fn download(url: &Url) -> Result<Html, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url.as_str()).await?.error_for_status()?;
    let document = resp.text().await?;
    let document = Html::parse_document(&document);
    info!("document downloaded");
    Ok(document)
}
