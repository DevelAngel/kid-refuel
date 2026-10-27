mod download;
mod config;
mod error;
mod load;
mod parse;
mod save;

mod collection;

use crate::download::*;
use crate::config::{FuelType, SourceConfig};
use crate::load::*;
use crate::parse::*;
use crate::save::*;

use refuel_app::prelude::{App, CollectionDiagnostic, record_collection_diagnostic};
use refuel_app::prelude::set_fallback_position;
use refuel_app::shell;
use refuel_db::*;

use leptos::prelude::get_configuration;
use leptos::prelude::*;
use rand::prelude::*;

use axum::Router;
use clap::{Parser, Subcommand};

use leptos_axum::{LeptosRoutes, generate_route_list};
use std::path::PathBuf;
use chrono_tz::Tz;
use url::Url;
use tokio::signal;
use tokio::sync::watch;
use tokio::time::{self, Duration};
use tower_http::trace::TraceLayer;

use tracing_subscriber::EnvFilter;

use tracing::{debug, info, warn};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(flatten)]
    common: SourceConfig,
}


#[derive(Subcommand)]
enum Commands {
    /// Download only mode
    Download {
        #[clap(flatten)]
        common: SourceConfig,
        #[arg(short, long, value_name = "FILE")]
        /// Filename of downloaded html document
        out: Option<PathBuf>,
    },
    /// Normal mode but only one run
    RunSingle {
        #[clap(flatten)]
        common: SourceConfig,
        #[arg(short, long, value_name = "FILE")]
        /// Use downloaded html document
        downloaded: Option<PathBuf>,
        #[arg(long)]
        /// do not save to database
        dry_run: bool,
    },
    /// Normal mode
    Run {
        #[clap(flatten)]
        common: SourceConfig,
        #[arg(long)]
        /// do not save to database
        dry_run: bool,
    },
}

fn calc_duration<R: Rng>(rng: &mut R, interval: &Duration) -> Duration {
    let var = rng.gen_range(0..=(10 * 60)); // 0 .. 10min
    let var = Duration::from_secs(var);

    if rng.gen_bool(0.5) {
        interval.saturating_add(var)
    } else {
        interval.saturating_sub(var)
    }
}

#[tracing::instrument(skip(url))]
async fn cmd_download(
    url: &Url,
    filename: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let document = download(url).await?;
    if let Some(filename) = filename.as_ref() {
        save_file(&document, filename).await?;
    } else {
        save_stdout(&document).await?;
    }
    Ok(())
}

#[tracing::instrument(skip(url))]
async fn cmd_run_single(
    url: &Url,
    source_time_zone: Tz,
    fuel_type: FuelType,
    downloaded: &Option<PathBuf>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let started_at = chrono::Local::now().to_rfc3339();
    let started = std::time::Instant::now();
    let (document, body, http_status, http_duration) = if let Some(downloaded) = downloaded {
        let document = load_file(downloaded).await?;
        let body = document.html();
        (document, body, None, None)
    } else {
        match download_with_details(url).await {
            Ok(page) => (
                page.document,
                page.body,
                Some(page.status),
                Some(page.duration),
            ),
            Err(error) => {
                record_collection_diagnostic(CollectionDiagnostic {
                    started_at,
                    source_url: url.clone(),
                    http_status: None,
                    http_duration: None,
                    duration: started.elapsed(),
                    result: "download_failed".to_owned(),
                    prices_found: 0,
                    rejected_entries: 0,
                    error: Some(error.to_string()),
                    response_excerpt: None,
                });
                return Err(error);
            }
        }
    };

    if let Some(status) = http_status.filter(|status| !status.is_success()) {
        let error = format!("HTTP request failed with status {status}");
        record_collection_diagnostic(CollectionDiagnostic {
            started_at,
            source_url: url.clone(),
            http_status,
            http_duration,
            duration: started.elapsed(),
            result: "http_failed".to_owned(),
            prices_found: 0,
            rejected_entries: 0,
            error: Some(error.clone()),
            response_excerpt: CollectionDiagnostic::response_excerpt(&body),
        });
        return Err(std::io::Error::other(error).into());
    }

    let parsed = match parse(&document, source_time_zone, fuel_type).await {
        Ok(parsed) => parsed,
        Err(error) => {
            record_collection_diagnostic(CollectionDiagnostic {
                started_at,
                source_url: url.clone(),
                http_status,
                http_duration,
                duration: started.elapsed(),
                result: "parse_failed".to_owned(),
                prices_found: 0,
                rejected_entries: 0,
                error: Some(error.to_string()),
                response_excerpt: CollectionDiagnostic::response_excerpt(&body),
            });
            return Err(error.into());
        }
    };

    let rejected_entries = parsed.rejected_entries;
    let refuel_stations = parsed.stations;
    let fetched = refuel_stations.len();
    if dry_run {
        for rs in refuel_stations.iter() {
            let name = &rs.station.name;
            let addr = &rs.station.addr;
            let updated = &rs.price_change.updated;
            let price = &rs.price_change.price;
            let price = price[0] as u16 * 1000 + price[1] as u16 * 10 + price[2] as u16;
            let price = price as f32 / 1000f32;
            debug!("name: {name}, addr: {addr}, updated: {updated}, price: {price:.3}");
        }
        info!("prices fetched: {fetched}");
        warn!("price changes not saved (dry run)");
    } else {
        info!("prices fetched: {fetched}");
        let mut conn = establish_connection_sqlite()?;
        let refuel_stations: Vec<_> = refuel_stations
            .into_iter()
            .map(|station| station.save(&mut conn))
            .collect::<Result<_, _>>()?;
        for rs in refuel_stations.iter() {
            let id = &rs.id;
            let name = &rs.station.name;
            let addr = &rs.station.addr;
            let updated = &rs.price_change.updated;
            let price = &rs.price_change.price;
            let price = price[0] as u16 * 1000 + price[1] as u16 * 10 + price[2] as u16;
            let price = price as f32 / 1000f32;
            debug!("id: {id}, name: {name}, addr: {addr}, updated: {updated}, price: {price:.3}");
        }
    }

    record_collection_diagnostic(CollectionDiagnostic {
        started_at,
        source_url: url.clone(),
        http_status,
        http_duration,
        duration: started.elapsed(),
        result: if fetched == 0 {
            "empty_result"
        } else if rejected_entries > 0 {
            "partial_success"
        } else {
            "success"
        }
        .to_owned(),
        prices_found: fetched,
        rejected_entries,
        error: if fetched == 0 {
            Some("Parser returned no prices".to_owned())
        } else if rejected_entries > 0 {
            Some(format!("{rejected_entries} invalid entries were skipped"))
        } else {
            None
        },
        response_excerpt: (fetched == 0 || rejected_entries > 0)
            .then(|| CollectionDiagnostic::response_excerpt(&body))
            .flatten(),
    });

    Ok(())
}

#[tracing::instrument(skip(url))]
async fn cmd_run_loop(url: &Url, dry_run: bool, source_time_zone: Tz, fuel_type: FuelType) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let interval = Duration::from_secs(60 * 60); // 60 min
    let mut first_run = true;
    loop {
        match cmd_run_single(url, source_time_zone, fuel_type, &None, dry_run).await {
            Ok(()) => {}
            Err(error) if first_run => return Err(error),
            Err(error) => tracing::error!("scheduled collection failed; continuing: {error}"),
        }
        first_run = false;

        let sleep_time = calc_duration(&mut rng, &interval); // 50min .. 70min
        info!("sleep for {:.2} min..", sleep_time.as_secs_f32() / 60.0);

        let mut shutdown = false;
        tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("CTRL+C pressed -> shutdown..");
                shutdown = true;
            }
            _ = time::sleep(sleep_time) => {}
        }
        if shutdown {
            break;
        }
    }
    info!("graceful shutdown");
    Ok(())
}

fn app() -> impl IntoView {
    view! { <App /> }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let cli = Cli::parse();
    if let Some(command) = &cli.command {
        match command {
            Commands::Download { common, out } => cmd_download(&common.url, out).await?,
            Commands::RunSingle {
                common,
                downloaded,
                dry_run,
            } => cmd_run_single(&common.url, common.source_time_zone, common.fuel_type, downloaded, dry_run.to_owned()).await?,

            Commands::Run { common, dry_run } => {
                cmd_run_loop(&common.url, dry_run.to_owned(), common.source_time_zone, common.fuel_type).await?
            }
        }
    } else {
        let fallback_position = cli.common.fallback_position.ok_or_else(|| {
            std::io::Error::other("missing fallback position; provide --fallback-position LATITUDE,LONGITUDE")
        })?;
        set_fallback_position(fallback_position).map_err(|_| {
            std::io::Error::other("fallback position was already configured")
        })?;

        let conf = get_configuration(None)?;
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(app);
        let shell_options = leptos_options.clone();

        let app = Router::new()
            .leptos_routes(&leptos_options, routes, move || {
                shell(shell_options.clone())
            })
            .fallback(leptos_axum::file_and_error_handler(shell))
            .layer(TraceLayer::new_for_http())
            .with_state(leptos_options);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("listening on http://{}", &addr);

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let mut server_shutdown = shutdown_rx.clone();
        let mut server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    while !*server_shutdown.borrow() {
                        if server_shutdown.changed().await.is_err() {
                            break;
                        }
                    }
                })
                .await
        });

        if let Err(error) = cmd_run_single(&cli.common.url, cli.common.source_time_zone, cli.common.fuel_type, &None, false).await {
            tracing::error!("initial collection failed; web server remains available: {error}");
        }

        let url = cli.common.url.clone();
        let source_time_zone = cli.common.source_time_zone;
        let fuel_type = cli.common.fuel_type;
        let collector = collection::run_schedule(
            move || {
                let url = url.clone();
                async move { cmd_run_single(&url, source_time_zone, fuel_type, &None, false).await }
            },
            shutdown_rx,
            Duration::from_secs(60 * 60),
            Duration::from_secs(10 * 60),
            source_time_zone,
        );
        tokio::pin!(collector);

        tokio::select! {
            result = signal::ctrl_c() => {
                result?;
                info!("CTRL+C pressed; shutting down");
            }
            result = &mut server => {
                result??;
                let _ = shutdown_tx.send(true);
                collector.await;
                return Ok(());
            }
            _ = &mut collector => {
                let _ = shutdown_tx.send(true);
                server.await??;
                return Ok(());
            }
        }

        let _ = shutdown_tx.send(true);
        collector.await;
        server.await??;
    }

    Ok(())
}
