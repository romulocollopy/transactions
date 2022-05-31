# Transactions

A simple rust program that reads transactions from a file ans writes to stdout.
The output can be directed to a file, as expected to a unix program.

```shell
$ cargo run -- transactions.csv > accounts.csv
```

The expected input is in the format:
```csv
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1, 4
resolve, 1, 4
dispute, 1, 3
withdrawal, 2, 5, 3.0
chargeback, 1, 3
```

And the output:
```csv
client,avaliable,held,total,locked
1,1.0,0,1.0,true
2,-1,0,-1,false
```

## Business Rules

The business rules are described in the [tests](https://github.com/romulocollopy/transactions/blob/main/src/domain/account.rs#L202)
