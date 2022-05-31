use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub type ClientID = u16;
pub type TransactionID = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionType {
    Withdraw(Decimal),
    Deposit(Decimal),
    Dispute,
    Resolve,
    ChargeBack,
}

// Transaction is a financial transaction representation
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub client: ClientID,
    pub kind: TransactionType,
    pub tx: TransactionID,
}

impl Transaction {
    pub fn create_deposit(
        client: ClientID,
        tx: TransactionID,
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
        tx: TransactionID,
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

    pub fn create_dispute(client: ClientID, tx: TransactionID) -> Result<Self, &'static str> {
        Ok(Self {
            client,
            tx,
            kind: TransactionType::Dispute,
        })
    }

    pub fn create_resolve(client: ClientID, tx: TransactionID) -> Result<Self, &'static str> {
        Ok(Self {
            client,
            tx,
            kind: TransactionType::Resolve,
        })
    }

    pub fn create_chargeback(client: ClientID, tx: TransactionID) -> Result<Self, &'static str> {
        Ok(Self {
            client,
            tx,
            kind: TransactionType::ChargeBack,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

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
}
