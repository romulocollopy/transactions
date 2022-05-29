use std::{env::args, process::exit};
use transactions_handler::reader::get_filename;
use transactions_handler::run;

fn main() {
    let arguments = args().collect::<Vec<String>>();
    let filename = get_filename(arguments).unwrap_or_else(|err| {
        eprintln!("Error getting filename: {}", err);
        exit(1);
    });

    run(filename)
}
