mod domain;
pub mod reader;
pub mod writer;

use reader::{get_content, get_reader};
use writer::{write, write_headers};

/// Application runner
///
/// Receives a String representing the filename of a CSV containing
/// a series of transactions, and processes the payments crediting and debiting accounts.
/// After processing the complete set of payments output the client account balances
///
/// ```
/// let result = transactions_handler::run(String::from("tests/transactions.csv"));
/// assert_eq!(result, ());
/// ```
pub fn run(filename: String) {
    let mut rdr = get_reader(filename);
    let mut portfolio = get_content(&mut rdr).unwrap();

    write_headers();
    loop {
        match portfolio.get_snapshot_line() {
            Some(s) => write(s),
            _ => {
                break;
            }
        }
    }
}
