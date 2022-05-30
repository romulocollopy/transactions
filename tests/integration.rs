use transactions_handler;

#[test]
fn test_run() {
    transactions_handler::run(String::from("tests/transactions.csv"));
}
