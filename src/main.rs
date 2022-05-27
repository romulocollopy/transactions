use std::{env::args, process::exit};
use transactions_handler::run;

fn main() {
    run(get_filename());
}

fn get_filename() -> String {
    let arguments = args().collect::<Vec<String>>();

    if arguments.len() != 2 {
        eprintln!("Wrong number of arguments: {:?}", arguments);
        exit(1)
    }
    arguments.get(1).unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::get_filename;

    #[test]
    fn wrong_args_number() {
        assert_eq!(get_filename(), "");
    }
}
