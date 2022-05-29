mod domain;
pub mod reader;

/// Application runner
///
/// Receives a String representing the filename of a CSV containing
/// a series of transactions, and processes the payments crediting and debiting accounts.
/// After processing the complete set of payments output the client account balances
///
/// ```
/// let result = transactions_handler::run(String::from("myfile.csv"));
/// assert_eq!(result, ());
/// ```
pub fn run(filename: String) {
    println!("Opening \"{}\"", filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        assert_eq!(run(String::from("myfile.csv")), ());
    }
}
