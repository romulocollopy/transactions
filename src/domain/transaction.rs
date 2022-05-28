use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, PartialEq, Clone)]
enum TransactionType {
    Withdraw(Decimal),
    Deposit(Decimal),
    Dispute,
    Resolve,
    ChargeBack,
}

// Transaction is a financial transaction representation
#[derive(Debug)]
pub struct Transaction {
    client: u16,
    kind: TransactionType,
    tx: u32,
}

impl Transaction {
    fn create_deposit(client: u16, tx: u32, amount: Decimal) -> Self {
        Self {
            client,
            tx,
            kind: TransactionType::Deposit(amount),
        }
    }

    fn create_withdraw(client: u16, tx: u32, amount: Decimal) -> Self {
        Self {
            client,
            tx,
            kind: TransactionType::Withdraw(amount),
        }
    }
}

#[derive(Debug)]
pub struct Account {
    client: u16,
    avaliable: Decimal,
    held: Decimal,
    locked: bool,
    transactions: Vec<Transaction>,
}

impl Account {
    fn add_transaction(&mut self, t: Transaction) -> Result<(), &str> {
        if self.client != t.client {
            return Err("Invalid transaction client for this account");
        }

        self.transactions.push(t);
        Ok(())
    }

    fn process_transactions(&mut self) -> Result<(), &str> {
        for t in self.transactions.iter() {
            match t.kind {
                TransactionType::Deposit(amount) => {
                    self.avaliable += amount;
                }
                TransactionType::Withdraw(amount) => {
                    self.avaliable -= amount;
                }
                TransactionType::Dispute => {}
                _ => return Err("Not Implemented"),
            };
        }

        Ok(())
    }

    fn get_total(&self) -> Decimal {
        self.avaliable + self.held
    }

    fn new(client: u16) -> Self {
        Self {
            client,
            avaliable: dec!(0),
            held: dec!(0),
            locked: false,
            transactions: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_deposit_to_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_deposit(2, 5, amount.clone());
        let mut account = Account::new(2);
        assert_eq!(account.get_total(), dec!(0));

        account.add_transaction(t).unwrap();
        account.process_transactions().unwrap();
        assert_eq!(account.get_total(), amount)
    }

    #[test]
    fn test_withdraw_from_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(2, 5, amount.clone());
        let mut account = Account::new(2);
        assert_eq!(account.get_total(), dec!(0));

        account.add_transaction(t).unwrap();
        account.process_transactions().unwrap();
        assert_eq!(account.get_total(), amount * dec!(-1))
    }

    #[test]
    fn test_mismatching_client() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(999, 5, amount.clone());
        let mut account = Account::new(2);

        assert_eq!(
            account.add_transaction(t),
            Err("Invalid transaction client for this account")
        );
    }

    #[test]
    fn test_new_transaction() {
        let client = 3;
        let kind = TransactionType::Withdraw(dec!(15.33));
        let tx = 12;

        let t = Transaction {
            client,
            kind: kind.clone(),
            tx,
        };

        assert_eq!(t.client, client);
        assert_eq!(t.kind, kind);
        assert_eq!(t.tx, tx);
    }

    #[test]
    fn test_new_account() {
        let client = 3;
        let a = Account::new(client);
        assert_eq!(a.client, client);
        assert_eq!(a.avaliable, dec!(0));
        assert_eq!(a.held, dec!(0));
        assert_eq!(a.locked, false);
    }

    #[test]
    fn test_get_total() {
        let a = Account {
            client: 3,
            avaliable: dec!(12),
            held: dec!(5),
            locked: false,
            transactions: vec![],
        };
        assert_eq!(a.get_total(), a.avaliable + a.held)
    }
}
