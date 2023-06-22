use std::net::SocketAddr;
use std::sync::Arc;

use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::{extract::State, routing::get, Router};
use axum_macros::debug_handler;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio::try_join;
use chrono::{DateTime, Local};

use tracing_subscriber::EnvFilter;

use tracing::{info, error};


#[derive(Clone)]
struct RefuelStationData {
    name: String,
    addr: String,
    updated: DateTime<Local>,
    price: [u8; 3],
}

impl RefuelStationData {
    fn new(name: &str, addr: &str, price: [u8; 3]) -> Self {
        let name = name.to_owned();
        let addr = addr.to_owned();
        let updated = Local::now();
        Self { name, addr, updated, price }
    }

    fn update(&mut self, price: [u8; 3]) {
        let updated = Local::now();
        self.price = price;
        self.updated = updated;
    }
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    price_list: &'a [RefuelStationData],
}

#[derive(Clone)]
struct AppState {
    data: Vec<RefuelStationData>
}

impl Default for AppState {
    fn default() -> Self {
        let data = vec![
            RefuelStationData::new("MyESSO", "Marienfelder Chaussee 171, 12349 Berlin", [1, 78, 9]),
            RefuelStationData::new("MyJET", "Rhinstr. 240, 13055 Berlin", [1, 79, 8]),
            RefuelStationData::new("MyTotalEnergies", "Landsberger Allee 376, 12681 Berlin", [1, 81, 9]),
            RefuelStationData::new("MyAGIP ENI", "Dietzgenstr. 127, 13158 Berlin", [1, 80, 9]),
            RefuelStationData::new("MyHEM", "Wittestr. 16, 13509 Berlin", [1, 76, 9]),
            RefuelStationData::new("MySTAR", "Prenzlauer Promenade 72-73, 13089 Berlin", [1, 77, 9]),
            RefuelStationData::new("MySHELL", "Bundesallee 200, 10717 Berlin", [1, 82, 9]),
            RefuelStationData::new("MyJET", "Storkower Straße 126-130, 10407 Berlin", [1, 76, 9]),
        ];
        Self { data }
    }
}

#[debug_handler]
async fn home(State(state): State<Arc<RwLock<AppState>>>) -> Response {
    let state = state.read().await;
    let data = &state.data;
    HomeTemplate { price_list: &data }.into_response()
}

async fn change_state(state: Arc<RwLock<AppState>>) -> Result<(), hyper::Error> {
    let mut i = 0;
    loop {
        sleep(Duration::from_secs(60)).await;
        let mut state = state.write().await;
        let len = state.data.len() as u8;
        let index: usize = (i % len).into();
        state.data[index].update([1, 75 + i, 9]);
        info!("{} updated", state.data[index].name);
        i = (i + 1) % (len * 2);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let state = AppState::default();
    let state = Arc::new(RwLock::new(state));

    let app = Router::new()
        .route("/", get(home))
        .with_state(Arc::clone(&state));

    let addr: SocketAddr = "127.0.0.1:8080".parse().expect("invalid socket address");
    info!("listening on http://{}", addr.to_string());
    let service = axum::Server::bind(&addr)
        .serve(app.into_make_service());

    let change_state = change_state(Arc::clone(&state));

    match try_join!(service, change_state) {
        Ok(_) => {info!("good bye");}
        Err(_) => {error!("bah, good bye");}
    }

    Ok(())
}
