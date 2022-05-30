use csv::Reader;
use csv::{ReaderBuilder, Trim};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{error::Error, fs::File, io};

use crate::domain::transaction::{Portfolio, Transaction};

#[derive(Debug, Deserialize)]
struct TransactionRow {
    r#type: String,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

pub fn get_reader(filename: String) -> Reader<File> {
    ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_path(filename)
        .unwrap()
}

pub fn get_content<R>(rdr: &mut Reader<R>) -> Result<(), Box<dyn Error>>
where
    R: io::Read,
{
    let mut portfolio = Portfolio::new();
    for result in rdr.deserialize() {
        let record: TransactionRow = result?;
        match record.r#type.as_str() {
            "deposit" => {
                let t =
                    Transaction::create_deposit(record.client, record.tx, record.amount.unwrap())
                        .unwrap();
                portfolio.add_transaction(t);
            }
            "withdrawal" => {
                let t =
                    Transaction::create_withdraw(record.client, record.tx, record.amount.unwrap())
                        .unwrap();
                portfolio.add_transaction(t);
            }
            "dispute" => {
                let t = Transaction::create_dispute(record.client, record.tx).unwrap();
                portfolio.add_transaction(t);
            }

            "chargeback" => {
                let t = Transaction::create_chargeback(record.client, record.tx).unwrap();
                portfolio.add_transaction(t);
            }

            "resolve" => {
                let t = Transaction::create_resolve(record.client, record.tx).unwrap();
                portfolio.add_transaction(t);
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn get_filename(arguments: Vec<String>) -> Result<String, &'static str> {
    if arguments.len() != 2 {
        return Err("Wrong number of arguments");
    }
    Ok(arguments.get(1).unwrap().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::ReaderBuilder;

    const DATA: &str = "\
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1, 4
resolve, 1, 4
dispute, 1, 3
withdrawal, 2, 5, 3.0
chargeback, 1, 3";

    #[test]
    fn test_get_content() {
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_reader(DATA.as_bytes());
        let result = get_content(&mut rdr).unwrap();
    }

    #[test]
    fn test_get_filename_from_args() {
        assert_eq!(
            get_filename(vec![String::from("bin"), String::from("filename.csv")]).unwrap(),
            String::from("filename.csv")
        );
    }

    #[test]
    fn wrong_args_number_3() {
        match get_filename(vec![
            String::from("bin"),
            String::from("filename.csv"),
            String::from("extra_arg"),
        ]) {
            Err(err) => {
                assert_eq!(err, "Wrong number of arguments")
            }
            _ => panic!("error expected"),
        }
    }

    #[test]
    fn wrong_args_number_1() {
        match get_filename(vec![String::from("bin")]) {
            Err(err) => {
                assert_eq!(err, "Wrong number of arguments")
            }
            _ => panic!("error expected"),
        }
    }
}
