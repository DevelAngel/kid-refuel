use crate::types::StationPriceChange;

use leptos::prelude::*;

#[server(endpoint = "get_price_comparison")]
pub async fn get_price_comparison(
    station_ids: Vec<i32>,
) -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn =
        establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_window(&station_ids, chrono::Utc::now(), &mut conn)
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter().map(StationPriceChange::from).collect();
    Ok(list)
}
