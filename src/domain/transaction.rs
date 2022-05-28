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
    total: Decimal,
    locked: bool,
}

impl Account {
    fn add_transaction(&mut self, t: Transaction) -> Result<(), &str> {
        if self.client != t.client {
            return Err("Invalid transaction client for this account");
        }
        match t.kind {
            TransactionType::Deposit(amount) => {
                self.total += amount;
                Ok(())
            }
            TransactionType::Withdraw(amount) => {
                self.total -= amount;
                Ok(())
            }
            _ => Err("Not Implemented"),
        }
    }

    fn new(client: u16) -> Self {
        Self {
            client,
            avaliable: dec!(0),
            held: dec!(0),
            total: dec!(0),
            locked: false,
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
        assert_eq!(account.total, dec!(0));

        account.add_transaction(t).unwrap();
        assert_eq!(account.total, amount)
    }

    #[test]
    fn test_withdraw_from_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(2, 5, amount.clone());
        let mut account = Account::new(2);
        assert_eq!(account.total, dec!(0));

        account.add_transaction(t).unwrap();
        assert_eq!(account.total, amount * dec!(-1))
    }

    #[test]
    fn test_mismatching_client() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(999, 5, amount.clone());
        let mut account = Account::new(2);
        assert_eq!(account.total, dec!(0));

        assert_eq!(
            account.add_transaction(t),
            Err("Invalid transaction client for this account")
        );
    }

    #[test]
    fn test_new_transaction_hash() {
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
    fn test_new_balance() {
        let client = 3;
        let avaliable = dec!(12);
        let total = dec!(12);
        let held = dec!(15.33);
        let locked = true;
        let b = Account {
            client,
            avaliable,
            held,
            total,
            locked,
        };

        assert_eq!(b.client, client);
        assert_eq!(b.avaliable, avaliable);
        assert_eq!(b.total, total);
        assert_eq!(b.held, held);
        assert_eq!(b.locked, locked);
    }
}
