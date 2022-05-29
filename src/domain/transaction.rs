use std::collections::HashMap;
use std::error::Error;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

type ClientID = u16;

pub struct Portfolio {
    accounts: HashMap<ClientID, Account>,
}

impl Portfolio {
    pub fn add_transaction(&mut self, t: Transaction) -> Result<(), Box<dyn Error>> {
        let client: ClientID = t.client;

        match self.accounts.get_mut(&client) {
            Some(account) => {
                account.add_transaction(t).unwrap();
                Ok(())
            }
            None => {
                let mut account = Account::new(client);
                account.add_transaction(t).unwrap();
                self.accounts.insert(client, account);
                Ok(())
            }
        }
    }

    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum TransactionType {
    Withdraw(Decimal),
    Deposit(Decimal),
    Dispute,
    Resolve,
    ChargeBack,
}

// Transaction is a financial transaction representation
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    client: ClientID,
    kind: TransactionType,
    tx: u32,
}

impl Transaction {
    pub fn create_deposit(
        client: ClientID,
        tx: u32,
        amount: Decimal,
    ) -> Result<Self, &'static str> {
        if amount < dec!(0) {
            return Err("Amount must be positive");
        }
        Ok(Self {
            client,
            tx,
            kind: TransactionType::Deposit(amount),
        })
    }

    pub fn create_withdraw(
        client: ClientID,
        tx: u32,
        amount: Decimal,
    ) -> Result<Self, &'static str> {
        if amount < dec!(0) {
            return Err("Amount must be positive");
        }
        Ok(Self {
            client,
            tx,
            kind: TransactionType::Withdraw(amount),
        })
    }

    pub fn create_dispute(client: ClientID, tx: u32) -> Result<Self, &'static str> {
        Ok(Self {
            client,
            tx,
            kind: TransactionType::Dispute,
        })
    }

    pub fn create_resolve(client: ClientID, tx: u32) -> Result<Self, &'static str> {
        Ok(Self {
            client,
            tx,
            kind: TransactionType::Resolve,
        })
    }

    pub fn create_chargeback(client: ClientID, tx: u32) -> Result<Self, &'static str> {
        Ok(Self {
            client,
            tx,
            kind: TransactionType::ChargeBack,
        })
    }
}

struct Snapshot {
    client: ClientID,
    total: Decimal,
    held: Decimal,
    locked: bool,
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

    fn get_available(&self) -> Decimal {
        self.total - self.held
    }
}

#[derive(Debug)]
struct Account {
    client: ClientID,
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
        let mut processing = self.transactions.clone();

        for t in self.transactions.iter() {
            match t.kind {
                TransactionType::Deposit(amount) => {
                    s.total += amount;
                }
                TransactionType::Withdraw(amount) => {
                    s.total -= amount;
                }
                TransactionType::Dispute => {
                    self.open_dispute(&t, &mut s);
                }
                TransactionType::ChargeBack => {
                    // If there is no dispute, ignore
                    if s.locked {
                        continue;
                    }

                    if let Some(disp) = Self::get_disputed_transaction(&t, &processing) {
                        self.apply_changeback(disp, &mut s).unwrap();
                        processing.remove(
                            processing
                                .iter()
                                .position(|x| x == disp)
                                .expect("Dispute not found"),
                        );
                    }
                }
                TransactionType::Resolve => {
                    // If there is no dispute, ignore
                    if let Some(disp) = Self::get_disputed_transaction(&t, &processing) {
                        self.resolve(disp, &mut s).unwrap();
                        processing.remove(
                            processing
                                .iter()
                                .position(|x| x == disp)
                                .expect("Dispute not found"),
                        );
                    }
                }
            }
        }

        return Ok(s);
    }

    fn apply_changeback(&self, t: &Transaction, s: &mut Snapshot) -> Result<(), &str> {
        let amount = match t.kind {
            TransactionType::Deposit(amount) => Ok(amount),
            TransactionType::Withdraw(amount) => Ok(amount),
            _ => Err("Only Withdraw and Deposit can be changed back"),
        }
        .unwrap();

        s.total -= amount;
        s.held -= amount;
        s.locked = true;
        Ok(())
    }

    fn resolve(&self, t: &Transaction, s: &mut Snapshot) -> Result<(), &str> {
        match t.kind {
            TransactionType::Deposit(amount) => {
                s.held -= amount;
                Ok(())
            }
            TransactionType::Withdraw(amount) => {
                s.held -= amount;
                Ok(())
            }
            _ => Err("Only Withdraw and Deposit can be changed back"),
        }
    }

    fn open_dispute(&self, t: &Transaction, s: &mut Snapshot) {
        for r in self.transactions.iter() {
            if r == t || r.tx != t.tx {
                continue;
            }
            match r.kind {
                TransactionType::Deposit(amount) => {
                    s.held += amount;
                }
                TransactionType::Withdraw(amount) => {
                    s.total += amount;
                    s.held += amount;
                }
                _ => continue,
            }
        }
    }

    fn get_disputed_transaction<'b>(
        cashback: &'b Transaction,
        processing_transactions: &'b Vec<Transaction>,
    ) -> Option<&'b Transaction> {
        let mut related_transactions: HashMap<&str, &Transaction> = HashMap::new();

        for r in processing_transactions.iter() {
            if r == cashback || r.tx != cashback.tx {
                continue;
            }

            match r.kind {
                TransactionType::Dispute => related_transactions.insert("dispute", r),
                TransactionType::Withdraw(_) => related_transactions.insert("reversed", r),
                TransactionType::Deposit(_) => related_transactions.insert("reversed", r),
                _ => continue,
            };
        }

        if let Some(_) = related_transactions.get("dispute") {
            if let Some(rev) = related_transactions.get("reversed") {
                return Some(*rev);
            };
        }
        None
    }

    fn new(client: ClientID) -> Self {
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(32.555));
        assert_eq!(s.held, dec!(30));

        account.add_transaction(chargeback).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), s.total);
        assert_eq!(s.total, dec!(32.555));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(chargeback2).unwrap();
        let s = account.take_snapshot().unwrap();
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(32.555));
        assert_eq!(s.held, dec!(30));

        account.add_transaction(chargeback).unwrap();
        let s = account.take_snapshot().unwrap();
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.total, dec!(15.7231));
        assert_eq!(s.held, dec!(5.7231));

        account.add_transaction(chargeback).unwrap();
        let s = account.take_snapshot().unwrap();
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(47.231));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.total, dec!(57.231));
        assert_eq!(s.get_available(), dec!(47.231));
        assert_eq!(s.held, dec!(10));

        account.add_transaction(resolve).unwrap();
        let s = account.take_snapshot().unwrap();
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(15.7231));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.total, dec!(15.7231));
        assert_eq!(s.get_available(), dec!(10));
        assert_eq!(s.held, dec!(5.7231));

        account.add_transaction(resolve).unwrap();
        let s = account.take_snapshot().unwrap();
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(47.2222));
        assert_eq!(s.total, dec!(47.2222));
        assert_eq!(s.held, dec!(0));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot().unwrap();
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
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(15.72));

        account.add_transaction(disp).unwrap();
        let s = account.take_snapshot().unwrap();
        assert_eq!(s.get_available(), dec!(10.00));
        assert_eq!(s.total, dec!(15.72));
        assert_eq!(s.held, dec!(5.72));
    }

    #[test]
    fn test_deposit_to_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_deposit(2, 5, amount.clone()).unwrap();
        let mut account = Account::new(2);
        assert_eq!(account.take_snapshot().unwrap().get_available(), dec!(0));

        account.add_transaction(t).unwrap();
        account.take_snapshot().unwrap();
        assert_eq!(account.take_snapshot().unwrap().get_available(), amount);
    }

    #[test]
    fn test_withdraw_from_account() {
        let amount = dec!(11.01);
        let t = Transaction::create_withdraw(2, 5, amount.clone()).unwrap();
        let mut account = Account::new(2);
        assert_eq!(account.take_snapshot().unwrap().get_available(), dec!(0));

        account.add_transaction(t).unwrap();
        assert_eq!(
            account.take_snapshot().unwrap().get_available(),
            amount * dec!(-1)
        );
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
            total: dec!(12),
            held: dec!(5),
            locked: false,
        };
        assert_eq!(s.get_available(), s.total - s.held)
    }
}
