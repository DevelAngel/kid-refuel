mod download;
mod error;
mod load;
mod parse;
mod save;

mod fallback;

use crate::download::*;
use crate::load::*;
use crate::parse::*;
use crate::save::*;

use crate::fallback::file_and_error_handler;

use refuel_app::*;
use refuel_db::*;

use leptos::*;
use rand::prelude::*;

use axum::{Router, routing::get};
use clap::{Parser, Subcommand, Args};
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::path::PathBuf;
use tokio::signal;
use tokio::time::{self, Duration};
use tower_http::trace::TraceLayer;
use url::Url;

use tracing_subscriber::EnvFilter;

use tracing::{warn, info, debug};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(flatten)]
    common: CommonArgs,
}

#[derive(Args)]
pub struct CommonArgs {
    #[arg(short, long, value_name = "URL", default_value_t = Url::parse("http://localhost:8080").unwrap())]
    /// Url of the webapp
    url: Url,
}

#[derive(Subcommand)]
enum Commands {
    /// Download only mode
    Download {
        #[clap(flatten)]
        common: CommonArgs,
        #[arg(short, long, value_name = "FILE")]
        /// Filename of downloaded html document
        out: Option<PathBuf>,
    },
    /// Normal mode but only one run
    RunSingle {
        #[clap(flatten)]
        common: CommonArgs,
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
        common: CommonArgs,
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
async fn cmd_download(url: &Url, filename: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let document = download(url).await?;
    if let Some(filename) = filename.as_ref() {
        save_file(&document, filename).await?;
    } else {
        save_stdout(&document).await?;
    }
    Ok(())
}

#[tracing::instrument(skip(url))]
async fn cmd_run_single(url: &Url, downloaded: &Option<PathBuf>, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let document = if let Some(downloaded) = downloaded {
        load_file(downloaded).await?
    } else {
        download(url).await?
    };

    let refuel_stations = parse(&document).await?;

    let conn = &mut establish_connection();

    let mut saved = 0;
    for rs in refuel_stations.iter() {
        let price = rs.price as f32 / 1000f32;
        if rs.save(conn) && !dry_run {
            saved += 1;
            debug!("name: {}, addr: {}, updated: {}, price: {:.3}", rs.name, rs.addr, rs.updated, price);
        } else if downloaded.is_some() || dry_run {
            // print all
            debug!("name: {}, addr: {}, updated: {}, price: {:.3}", rs.name, rs.addr, rs.updated, price);
        }
    }
    if dry_run {
        info!("prices fetched: {fetched}", fetched = refuel_stations.len());
        warn!("price changes not saved");
    } else {
        info!("price changes saved: {saved} / {fetched}", fetched = refuel_stations.len());
    }
    Ok(())
}

#[tracing::instrument(skip(url))]
async fn cmd_run_loop(url: &Url, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let interval = Duration::from_secs(20 * 60); // 20 min
    loop {
        cmd_run_single(url, &None, dry_run).await?;

        let sleep_time = calc_duration(&mut rng, &interval); // 10min .. 30min
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

fn app(cx: leptos::Scope) -> impl IntoView {
    use refuel_app::*;

    view! { cx, <App /> }
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
            Commands::Download { common, out } => { cmd_download(&common.url, out).await? }
            Commands::RunSingle { common, downloaded, dry_run } => { cmd_run_single(&common.url, downloaded, dry_run.to_owned()).await? }
            Commands::Run { common, dry_run } => { cmd_run_loop(&common.url, dry_run.to_owned()).await? }
        }
    } else {
        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;

        _ = GetCurrentPrices::register();

        // Generate the list of routes in your Leptos App
        let routes = generate_route_list(app).await;

        let leptos_routes = Router::new()
            .leptos_routes(
                leptos_options.clone(),
                routes,
                app
            );

        let other_routes = Router::new()
            .route("/api/*fn_name", get(leptos_axum::handle_server_fns).post(leptos_axum::handle_server_fns))
            .route("/favicon.ico", get(file_and_error_handler))
            .fallback(file_and_error_handler)
            .with_state(leptos_options);

        let app = Router::new()
            .merge(other_routes)
            .merge(leptos_routes)
            .layer(TraceLayer::new_for_http());

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        info!("listening on http://{}", &addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    Ok(())
}
