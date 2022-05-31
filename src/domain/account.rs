use std::{collections::HashMap, error::Error};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::transaction::{ClientID, Transaction, TransactionID, TransactionType};

#[derive(Debug)]
pub struct Portfolio {
    accounts: Vec<Account>,
    _pos: i32,
}

impl Portfolio {
    pub fn add_transaction(&mut self, t: Transaction) -> Result<(), Box<dyn Error>> {
        let client: ClientID = t.client;

        for account in self.accounts.iter_mut() {
            if account.client == client {
                account.add_transaction(t)?;
                return Ok(());
            }
        }

        let mut account = Account::new(client);
        account.add_transaction(t).unwrap();
        self.accounts.push(account);

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            accounts: vec![],
            _pos: 0,
        }
    }

    pub fn get_snapshot_line(&mut self) -> Option<Snapshot> {
        match self.accounts.get(self._pos as usize) {
            Some(account) => {
                self._pos += 1;
                Some(account.take_snapshot())
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Account {
    client: ClientID,
    transactions: Vec<Transaction>,
    disputed_transactions: HashMap<TransactionID, Transaction>,
    snapshot: Snapshot,
}

impl Account {
    fn add_transaction(&mut self, t: Transaction) -> Result<(), &str> {
        if self.client != t.client {
            return Err("Invalid transaction client for this account");
        }

        self.transactions.push(t.clone());

        match t.kind {
            TransactionType::Deposit(amount) => {
                self.snapshot.total += amount;
            }
            TransactionType::Withdraw(amount) => {
                self.snapshot.total -= amount;
            }
            TransactionType::Dispute => {
                self.open_dispute(t);
            }
            TransactionType::ChargeBack => {
                // If there is no dispute, ignore
                if self.snapshot.locked {
                    eprintln!("Cannot chargeback a locked account");
                    return Ok(());
                }

                if let Some(disp) = self.get_disputed_transaction(t) {
                    self.apply_changeback(disp).unwrap();
                }
            }
            TransactionType::Resolve => {
                // If there is no dispute, ignore
                if let Some(disp) = self.get_disputed_transaction(t) {
                    self.resolve(disp).unwrap();
                }
            }
        }
        // self.transactions.push(t);
        Ok(())
    }

    fn take_snapshot(&self) -> Snapshot {
        self.snapshot.clone()
    }

    fn apply_changeback(&mut self, disputed: Transaction) -> Result<(), &str> {
        let amount = match disputed.kind {
            TransactionType::Deposit(amount) => Ok(amount),
            TransactionType::Withdraw(amount) => Ok(amount),
            _ => Err("Only Withdraw and Deposit can be changed back"),
        }
        .unwrap();

        self.snapshot.total -= amount;
        self.snapshot.held -= amount;
        self.snapshot.locked = true;
        Ok(())
    }

    fn resolve(&mut self, disputed: Transaction) -> Result<(), &str> {
        match disputed.kind {
            TransactionType::Deposit(amount) => {
                self.snapshot.held -= amount;
                Ok(())
            }
            TransactionType::Withdraw(amount) => {
                self.snapshot.held -= amount;
                Ok(())
            }
            _ => Err("Only Withdraw and Deposit can be changed back"),
        }
    }

    fn open_dispute(&mut self, t: Transaction) {
        if let Some(_) = self.get_disputed_transaction(t.clone()) {
            eprintln!("Dispute for this transaction already open. Nothing to do.");
            return;
        };

        for r in self.transactions.iter() {
            if r == &t || r.tx != t.tx {
                continue;
            }

            let original = r.clone();

            match r.kind {
                TransactionType::Deposit(amount) => {
                    self.snapshot.held += amount;
                    self.disputed_transactions.insert(t.tx, original);
                }
                TransactionType::Withdraw(amount) => {
                    self.snapshot.total += amount;
                    self.snapshot.held += amount;
                    self.disputed_transactions.insert(t.tx, original);
                }
                _ => {
                    eprintln!("Invalid TX. Dispute can't be opened");
                    continue;
                }
            }
        }
    }

    fn get_disputed_transaction(&self, t: Transaction) -> Option<Transaction> {
        match self.disputed_transactions.get(&t.tx) {
            Some(disp) => Some(disp.clone()),
            None => None,
        }
    }

    fn new(client: ClientID) -> Self {
        Self {
            client,
            transactions: vec![],
            disputed_transactions: HashMap::new(),
            snapshot: Snapshot::new(client),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub client: ClientID,
    pub total: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

impl Snapshot {
    fn new(client: ClientID) -> Self {
        Self {
            client,
            total: dec!(0),
            held: dec!(0),
            locked: false,
        }
    }

    pub fn get_available(&self) -> Decimal {
        self.total - self.held
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_do_not_double_chargeback_withdraw() {
        let dep = Transaction::create_deposit(2, 1, dec!(62.555)).unwrap();
        let withdraw = Transaction::create_withdraw(2, 2, dec!(30.0000)).unwrap();
        let disp = Transaction::create_dispute(2, withdraw.tx).unwrap();
        let chargeback = Transaction::create_chargeback(2, withdraw.tx).unwrap();
        let chargeback2 = Transaction::create_chargeback(2, withdraw.tx).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep).unwrap();
        account.add_transaction(withdraw).unwrap();

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(32.555));
        assert_eq!(s.held, dec!(30));

        account.add_transaction(chargeback).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.total, dec!(32.555));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(chargeback2).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.total, dec!(32.555));
        assert_eq!(s.held, dec!(0));
    }

    #[test]
    fn test_chargeback_withdraw() {
        let dep = Transaction::create_deposit(2, 1, dec!(62.555)).unwrap();
        let withdraw = Transaction::create_withdraw(2, 2, dec!(30.0000)).unwrap();
        let disp = Transaction::create_dispute(2, withdraw.tx).unwrap();
        let chargeback = Transaction::create_chargeback(2, withdraw.tx).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep).unwrap();
        account.add_transaction(withdraw).unwrap();

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(32.555));
        assert_eq!(s.held, dec!(30));

        account.add_transaction(chargeback).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.total, dec!(32.555));
        assert_eq!(s.held, dec!(0));
    }

    #[test]
    fn test_chargeback_deposit() {
        let dep1 = Transaction::create_deposit(2, 1, dec!(5.7231)).unwrap();
        let dep2 = Transaction::create_deposit(2, 2, dec!(10.0000)).unwrap();
        let disp = Transaction::create_dispute(2, 1).unwrap();
        let chargeback = Transaction::create_chargeback(2, 1).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep1).unwrap();
        account.add_transaction(dep2).unwrap();

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.total, dec!(15.7231));
        assert_eq!(s.held, dec!(5.7231));

        account.add_transaction(chargeback).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.total, dec!(10.0000));
        assert_eq!(s.held, dec!(0));
    }

    #[test]
    fn test_resolve_withdraw() {
        let dep = Transaction::create_deposit(2, 1, dec!(57.231)).unwrap();
        let withdraw = Transaction::create_withdraw(2, 2, dec!(10)).unwrap();
        let disp = Transaction::create_dispute(2, withdraw.tx).unwrap();
        let resolve = Transaction::create_resolve(2, withdraw.tx).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep).unwrap();
        account.add_transaction(withdraw).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(47.231));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.total, dec!(57.231));
        assert_eq!(s.get_available(), dec!(47.231));
        assert_eq!(s.held, dec!(10));

        account.add_transaction(resolve).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.get_available(), dec!(57.231));
        assert_eq!(s.held, dec!(0));
    }

    #[test]
    fn test_resolve_deposit() {
        let dep1 = Transaction::create_deposit(2, 1, dec!(5.7231)).unwrap();
        let dep2 = Transaction::create_deposit(2, 2, dec!(10.0000)).unwrap();
        let disp = Transaction::create_dispute(2, dep1.tx).unwrap();
        let resolve = Transaction::create_resolve(2, dep1.tx).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep1).unwrap();
        account.add_transaction(dep2).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(15.7231));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.total, dec!(15.7231));
        assert_eq!(s.get_available(), dec!(10));
        assert_eq!(s.held, dec!(5.7231));

        account.add_transaction(resolve).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.get_available(), dec!(15.7231));
        assert_eq!(s.held, dec!(0));
    }

    #[test]
    fn test_open_dispute_withdraw() {
        let dep1 = Transaction::create_deposit(2, 1, dec!(57.2222)).unwrap();
        let withdraw = Transaction::create_withdraw(2, 2, dec!(10)).unwrap();
        let disp = Transaction::create_dispute(2, withdraw.tx).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep1).unwrap();
        account.add_transaction(withdraw).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(47.2222));
        assert_eq!(s.total, dec!(47.2222));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(47.2222));
        assert_eq!(s.total, dec!(57.2222));
        assert_eq!(s.held, dec!(10));
    }

    #[test]
    fn test_open_dispute_deposit() {
        let dep1 = Transaction::create_deposit(2, 1, dec!(5.72)).unwrap();
        let dep2 = Transaction::create_deposit(2, 2, dec!(10)).unwrap();
        let disp = Transaction::create_dispute(2, 1).unwrap();

        let mut account = Account::new(2);
        account.add_transaction(dep1).unwrap();
        account.add_transaction(dep2).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(15.72));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot();
        assert_eq!(s.get_available(), dec!(10.00));
        assert_eq!(s.total, dec!(15.72));
        assert_eq!(s.held, dec!(5.72));
    }

    #[test]
    fn test_deposit_to_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_deposit(2, 5, amount.clone()).unwrap();
        let mut account = Account::new(2);
        assert_eq!(account.take_snapshot().get_available(), dec!(0));

        account.add_transaction(t).unwrap();
        account.take_snapshot();
        assert_eq!(account.take_snapshot().get_available(), amount);
    }

    #[test]
    fn test_withdraw_from_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(2, 5, amount.clone()).unwrap();
        let mut account = Account::new(2);
        assert_eq!(account.take_snapshot().get_available(), dec!(0));

        account.add_transaction(t).unwrap();
        assert_eq!(account.take_snapshot().get_available(), amount * dec!(-1));
    }

    #[test]
    fn test_mismatching_client() {
        let t = Transaction::create_withdraw(999, 5, dec!(11.01)).unwrap();
        let mut account = Account::new(2);

        assert_eq!(
            account.add_transaction(t),
            Err("Invalid transaction client for this account")
        );
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
            total: dec!(12),
            held: dec!(5),
            locked: false,
        };
        assert_eq!(s.get_available(), s.total - s.held)
    }
}
