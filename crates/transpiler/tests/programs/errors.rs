use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq)]
enum BankError {
    InsufficientFunds { needed: u64, available: u64 },
    AccountNotFound(String),
    Frozen,
}

impl fmt::Display for BankError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BankError::InsufficientFunds { needed, available } => {
                write!(f, "need {} but only {} available", needed, available)
            }
            BankError::AccountNotFound(name) => write!(f, "no account named {}", name),
            BankError::Frozen => write!(f, "account frozen"),
        }
    }
}

struct Bank {
    accounts: HashMap<String, u64>,
    frozen: Vec<String>,
}

impl Bank {
    fn new() -> Self {
        Bank { accounts: HashMap::new(), frozen: Vec::new() }
    }

    fn deposit(&mut self, name: &str, amount: u64) {
        *self.accounts.entry(name.to_string()).or_insert(0) += amount;
    }

    fn balance(&self, name: &str) -> Result<u64, BankError> {
        self.accounts
            .get(name)
            .copied()
            .ok_or(BankError::AccountNotFound(name.to_string()))
    }

    fn withdraw(&mut self, name: &str, amount: u64) -> Result<u64, BankError> {
        if self.frozen.contains(&name.to_string()) {
            return Err(BankError::Frozen);
        }
        let available = self.balance(name)?;
        if available < amount {
            return Err(BankError::InsufficientFunds { needed: amount, available });
        }
        let left = available - amount;
        self.accounts.insert(name.to_string(), left);
        Ok(left)
    }

    fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), BankError> {
        self.withdraw(from, amount)?;
        self.deposit(to, amount);
        Ok(())
    }
}

fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let (a, b) = s.split_once(',')?;
    let a = a.trim().parse::<i32>().ok()?;
    let b = b.trim().parse::<i32>().ok()?;
    Some((a, b))
}

fn first_word_len(s: &str) -> usize {
    let Some(word) = s.split_whitespace().next() else {
        return 0;
    };
    word.len()
}

fn checked(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(format!("cannot divide {} by zero", a))
    } else {
        Ok(a / b)
    }
}

fn main() {
    let mut bank = Bank::new();
    bank.deposit("alice", 100);
    bank.deposit("bob", 20);
    match bank.withdraw("alice", 30) {
        Ok(left) => println!("alice has {left} left"),
        Err(e) => println!("error: {e}"),
    }
    match bank.withdraw("bob", 50) {
        Ok(left) => println!("bob has {left} left"),
        Err(e) => println!("error: {e} / {:?}", e),
    }
    if let Err(e) = bank.transfer("carol", "bob", 5) {
        println!("transfer failed: {}", e);
    }
    bank.frozen.push("bob".to_string());
    let r = bank.withdraw("bob", 1);
    println!("frozen: {:?}", r);
    println!("is_err: {}", bank.withdraw("bob", 1).is_err());
    bank.transfer("alice", "dave", 10).unwrap();
    println!("dave: {}", bank.balance("dave").unwrap_or(0));
    println!("eve: {}", bank.balance("eve").unwrap_or(0));

    println!("{:?} {:?} {:?}", parse_pair("3, 4"), parse_pair("x,1"), parse_pair("5"));
    println!("{} {}", first_word_len("hello world"), first_word_len("   "));
    println!("{:?} {:?}", checked(10, 2), checked(1, 0));
    let results: Vec<i32> = [4, 2, 1].iter().map(|d| checked(8, *d).unwrap()).collect();
    println!("{:?}", results);
    let n: i32 = "42".parse().unwrap();
    let bad = "4x2".parse::<i32>().is_err();
    println!("{n} {bad}");
    let maybe: Option<&str> = None;
    println!("{}", maybe.map(|s| s.len()).unwrap_or(7));
}
