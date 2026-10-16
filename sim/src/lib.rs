use askama::Template;
use chrono::{DateTime, Local};

#[derive(Clone)]
struct RefuelStationData {
    name: String,
    addr: String,
    updated: DateTime<Local>,
    price: [u8; 3],
}

impl RefuelStationData {
    fn new(name: &str, addr: &str, price: [u8; 3]) -> Self {
        Self {
            name: name.to_owned(),
            addr: addr.to_owned(),
            updated: Local::now(),
            price,
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    price_list: &'a [RefuelStationData],
}

pub fn render_home() -> Result<String, askama::Error> {
    let stations = [
        RefuelStationData::new(
            "MyESSO",
            "Marienfelder Chaussee 171, 12349 Berlin",
            [1, 78, 9],
        ),
        RefuelStationData::new("MyJET", "Rhinstr. 240, 13055 Berlin", [1, 79, 8]),
        RefuelStationData::new(
            "MyTotalEnergies",
            "Landsberger Allee 376, 12681 Berlin",
            [1, 81, 9],
        ),
        RefuelStationData::new("MyAGIP ENI", "Dietzgenstr. 127, 13158 Berlin", [1, 80, 9]),
        RefuelStationData::new("MyHEM", "Wittestr. 16, 13509 Berlin", [1, 76, 9]),
        RefuelStationData::new(
            "MySTAR",
            "Prenzlauer Promenade 72-73, 13089 Berlin",
            [1, 77, 9],
        ),
        RefuelStationData::new("MySHELL", "Bundesallee 200, 10717 Berlin", [1, 82, 9]),
        RefuelStationData::new("MyJET", "Storkower Straße 126-130, 10407 Berlin", [1, 76, 9]),
    ];

    HomeTemplate {
        price_list: &stations,
    }
    .render()
}
