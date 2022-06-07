use crate::domain::Snapshot;
use csv::WriterBuilder;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SnapshotRow {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

pub fn write_headers() {
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);

    let row = SnapshotRow {
        client: 0,
        total: dec!(0),
        held: dec!(0),
        available: dec!(0),
        locked: false,
    };
    wtr.serialize(row).unwrap();
    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    let vec: Vec<&str> = data.split("\n").collect();
    println!("{}", vec[0])
}

pub fn write(s: Snapshot) {
    let row = SnapshotRow {
        client: s.client,
        total: s.total,
        held: s.held,
        available: s.get_available(),
        locked: s.locked,
    };

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);
    wtr.serialize(row).unwrap();

    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    print!("{}", data)
}
