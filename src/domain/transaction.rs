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

    fn create_dispute(client: u16, tx: u32) -> Self {
        Self {
            client,
            tx,
            kind: TransactionType::Dispute,
        }
    }

    fn create_resolve(client: u16, tx: u32) -> Self {
        Self {
            client,
            tx,
            kind: TransactionType::Resolve,
        }
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        std::ptr::eq(self, other)
    }
}

struct Snapshot {
    client: u16,
    avaliable: Decimal,
    held: Decimal,
    locked: bool,
}

impl Snapshot {
    fn new(client: u16) -> Self {
        Self {
            client,
            avaliable: dec!(0),
            held: dec!(0),
            locked: false,
        }
    }

    fn get_total(&self) -> Decimal {
        self.avaliable + self.held
    }
}

#[derive(Debug)]
pub struct Account {
    client: u16,
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

    fn take_snapshot(&mut self) -> Result<Snapshot, &str> {
        let mut s = Snapshot::new(self.client);
        for t in self.transactions.iter() {
            match t.kind {
                TransactionType::Deposit(amount) => {
                    s.avaliable += amount;
                }
                TransactionType::Withdraw(amount) => {
                    s.avaliable -= amount;
                }
                TransactionType::Dispute => {
                    if !self.has_resolution(t) {
                        self.open_dispute(&t, &mut s).unwrap();
                    }
                }
                TransactionType::Resolve => {}
                _ => return Err("Not Implemented"),
            };
        }

        Ok(s)
    }

    fn open_dispute(&self, t: &Transaction, s: &mut Snapshot) -> Result<(), &str> {
        for r in self.transactions.iter() {
            if r == t || r.tx != t.tx {
                continue;
            }
            match r.kind {
                TransactionType::Deposit(amount) => {
                    s.avaliable -= amount;
                    s.held += amount;
                }
                TransactionType::Withdraw(amount) => {
                    s.avaliable -= amount;
                    s.held += amount;
                }
                _ => return Err("Only Withdraws and Deposits can be Disputed"),
            }
        }
        Ok(())
    }

    fn has_resolution(&self, t: &Transaction) -> bool {
        for r in self.transactions.iter() {
            if r == t || r.tx != t.tx {
                continue;
            }

            if let TransactionType::Resolve = r.kind {
                return true;
            }
        }
        return false;
    }

    fn new(client: u16) -> Self {
        Self {
            client,
            transactions: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_resolve() {
        let dep1 = Transaction::create_deposit(2, 1, dec!(5.7231));
        let dep2 = Transaction::create_deposit(2, 2, dec!(10.0000));
        let disp = Transaction::create_dispute(2, 1);
        let resolve = Transaction::create_resolve(2, 1);

        let mut account = Account::new(2);
        account.add_transaction(dep1).unwrap();
        account.add_transaction(dep2).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_total(), dec!(15.7231));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_total(), dec!(15.7231));
        assert_eq!(s.avaliable, dec!(10));
        assert_eq!(s.held, dec!(5.7231));

        account.add_transaction(resolve).unwrap();
        let s = account.take_snapshot().unwrap();
        assert!(s.get_total() == s.avaliable);
        assert_eq!(s.get_total(), dec!(15.7231));
        assert_eq!(s.held, dec!(0));
    }

    #[test]
    fn test_open_dispute() {
        let dep1 = Transaction::create_deposit(2, 1, dec!(5.72));
        let dep2 = Transaction::create_deposit(2, 2, dec!(10));
        let disp = Transaction::create_dispute(2, 1);

        let mut account = Account::new(2);
        account.add_transaction(dep1).unwrap();
        account.add_transaction(dep2).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_total(), dec!(15.72));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_total(), dec!(15.72));
        assert_eq!(s.avaliable, dec!(10));
        assert_eq!(s.held, dec!(5.72));
    }

    #[test]
    fn test_deposit_to_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_deposit(2, 5, amount.clone());
        let mut account = Account::new(2);
        assert_eq!(account.take_snapshot().unwrap().get_total(), dec!(0));

        account.add_transaction(t).unwrap();
        account.take_snapshot().unwrap();
        assert_eq!(account.take_snapshot().unwrap().get_total(), amount);
    }

    #[test]
    fn test_withdraw_from_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(2, 5, amount.clone());
        let mut account = Account::new(2);
        assert_eq!(account.take_snapshot().unwrap().get_total(), dec!(0));

        account.add_transaction(t).unwrap();
        assert_eq!(
            account.take_snapshot().unwrap().get_total(),
            amount * dec!(-1)
        );
    }

    #[test]
    fn test_mismatching_client() {
        let t = Transaction::create_withdraw(999, 5, dec!(11.01));
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
        assert_eq!(a.transactions.len(), 0);
    }

    #[test]
    fn test_get_total() {
        let s = Snapshot {
            client: 3,
            avaliable: dec!(12),
            held: dec!(5),
            locked: false,
        };
        assert_eq!(s.get_total(), s.avaliable + s.held)
    }
}
