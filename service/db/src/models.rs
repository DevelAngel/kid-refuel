use super::schema::price_changes;

use diesel::prelude::*;

use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Queryable)]
pub struct RefuelStationPriceChange {
    pub name: String,
    pub addr: String,
    pub updated: DateTime<Utc>,
    pub price: u16,
}

#[derive(Insertable)]
#[diesel(table_name = price_changes)]
struct NewRefuelStationPriceChange<'a> {
    name: &'a str,
    addr: &'a str,
    updated: NaiveDateTime,
    price: i32,
}

impl RefuelStationPriceChange {
    pub fn save(&self, conn: &mut SqliteConnection) -> bool {
        let new = NewRefuelStationPriceChange::from(self);
        new.insert(conn)
    }
}

impl<'a> NewRefuelStationPriceChange<'a> {
    pub(crate) fn insert(self, conn: &mut SqliteConnection) -> bool {
        use crate::schema::price_changes::dsl::*;

        let inserted = diesel::insert_into(price_changes)
            .values(self)
            .on_conflict_do_nothing()
            .execute(conn)
            .expect("Error saving new station");
        inserted > 0
    }
}

impl<'a> From<&'a RefuelStationPriceChange> for NewRefuelStationPriceChange<'a> {
    fn from(src: &'a RefuelStationPriceChange) -> Self {
        Self {
            name: &src.name,
            addr: &src.addr,
            updated: src.updated.naive_utc(),
            price: src.price.into(),
        }
    }
}
