use std::{env::args, process::exit};
use transactions_handler::run;

fn main() {
    let arguments = args().collect::<Vec<String>>();
    let filename = get_filename(arguments).unwrap_or_else(|err| {
        eprintln!("Error getting filename: {}", err);
        exit(1);
    });

    run(filename)
}

fn get_filename(arguments: Vec<String>) -> Result<String, &'static str> {
    if arguments.len() != 2 {
        return Err("Wrong number of arguments");
    }
    Ok(arguments.get(1).unwrap().to_owned())
}

#[cfg(test)]
mod tests {
    use super::get_filename;

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
