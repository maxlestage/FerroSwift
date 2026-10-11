use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Tx {
    Deposit(u64),
    Withdraw(u64),
    Transfer { to: String, amount: u64 },
}

#[derive(Debug, PartialEq)]
enum BankError {
    NoAccount(String),
    Insufficient { needed: u64, available: u64 },
    Overflow,
    Frozen,
}

impl fmt::Display for BankError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BankError::NoAccount(n) => write!(f, "no account named {n:?}"),
            BankError::Insufficient { needed, available } => write!(f, "need {needed}, have {available}"),
            BankError::Overflow => f.write_str("overflow"),
            BankError::Frozen => f.write_str("account frozen"),
        }
    }
}

#[derive(Debug, Default)]
struct Account {
    balance: u64,
    frozen: bool,
    history: Vec<Tx>,
}

#[derive(Default)]
struct Bank {
    accounts: HashMap<String, Account>,
}

impl Bank {
    fn open(&mut self, name: &str, initial: u64) -> &mut Account {
        let acct = self.accounts.entry(name.to_string()).or_insert_with(Account::default);
        acct.balance += initial;
        if initial > 0 {
            acct.history.push(Tx::Deposit(initial));
        }
        acct
    }

    fn get(&mut self, name: &str) -> Result<&mut Account, BankError> {
        self.accounts.get_mut(name).ok_or_else(|| BankError::NoAccount(name.to_string()))
    }

    fn deposit(&mut self, name: &str, amount: u64) -> Result<u64, BankError> {
        let acct = self.get(name)?;
        if acct.frozen {
            return Err(BankError::Frozen);
        }
        acct.balance = acct.balance.checked_add(amount).ok_or(BankError::Overflow)?;
        acct.history.push(Tx::Deposit(amount));
        Ok(acct.balance)
    }

    fn withdraw(&mut self, name: &str, amount: u64) -> Result<u64, BankError> {
        let acct = self.get(name)?;
        if acct.frozen {
            return Err(BankError::Frozen);
        }
        if acct.balance < amount {
            return Err(BankError::Insufficient { needed: amount, available: acct.balance });
        }
        acct.balance -= amount;
        acct.history.push(Tx::Withdraw(amount));
        Ok(acct.balance)
    }

    fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), BankError> {
        if !self.accounts.contains_key(to) {
            return Err(BankError::NoAccount(to.to_string()));
        }
        self.withdraw(from, amount)?;
        if let Some(last) = self.get(from)?.history.last_mut() {
            *last = Tx::Transfer { to: to.to_string(), amount };
        }
        self.deposit(to, amount)?;
        Ok(())
    }

    fn freeze_all_below(&mut self, limit: u64) -> usize {
        let mut n = 0;
        for acct in self.accounts.values_mut() {
            if acct.balance < limit && !acct.frozen {
                acct.frozen = true;
                n += 1;
            }
        }
        n
    }

    fn close_empty(&mut self) -> Vec<String> {
        let mut closed: Vec<String> = self.accounts.iter().filter(|(_, a)| a.balance == 0).map(|(n, _)| n.clone()).collect();
        closed.sort();
        self.accounts.retain(|_, a| a.balance > 0);
        closed
    }

    fn total(&self) -> u64 {
        self.accounts.values().map(|a| a.balance).sum()
    }

    fn report(&self) -> String {
        let mut names: Vec<&String> = self.accounts.keys().collect();
        names.sort();
        names
            .iter()
            .map(|n| {
                let a = &self.accounts[*n];
                let deposits: u64 = a.history.iter().filter_map(|t| if let Tx::Deposit(x) = t { Some(*x) } else { None }).sum();
                format!("{n:<6}{:>10.2}{}{deposits:>6}", a.balance as f64 / 100.0, if a.frozen { " *" } else { "  " })
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn main() {
    let mut bank = Bank::default();
    bank.open("ann", 10_000);
    bank.open("bob", 250).frozen = false;
    bank.open("cid", 0);
    println!("{:?} {:?}", bank.deposit("ann", 500), bank.withdraw("bob", 300));
    println!("{:?} {:?}", bank.withdraw("zed", 1), bank.transfer("ann", "bob", 4_000));
    println!("{:?} {:?}", bank.transfer("bob", "nobody", 1), bank.deposit("cid", u64::MAX));
    println!("{:?}", bank.deposit("cid", 1));
    println!("{:?}", bank.withdraw("cid", u64::MAX));
    println!("{} {}", bank.freeze_all_below(1_000), bank.deposit("cid", 5).unwrap_err());
    for e in [bank.withdraw("bob", 10_000).unwrap_err(), BankError::NoAccount("q".into()), BankError::Overflow] {
        println!("{e} / {e:?}");
    }
    println!("{:?} {}", bank.close_empty(), bank.total());
    println!("{}", bank.report());
    let ann = bank.get("ann").unwrap();
    println!("{:?}", ann.history);
    println!("{:?} {}", ann.history.iter().rev().find(|t| matches!(t, Tx::Transfer { .. })), ann.history.iter().filter(|t| matches!(t, Tx::Deposit(x) if *x > 400)).count());
    let bob_history = bank.get("bob").map(|a| a.history.clone()).unwrap_or_default();
    println!("{:?} {}", bob_history, bob_history.contains(&Tx::Deposit(4_000)));
    let as_i64: Vec<i64> = bank.accounts.values().map(|a| i64::try_from(a.balance).unwrap_or(-1)).collect::<Vec<_>>();
    let mut as_i64 = as_i64;
    as_i64.sort_unstable();
    println!("{:?} {:?}", as_i64, u64::try_from(-5i64).is_err());
}
