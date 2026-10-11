// Translated with `--overflow-checks`: the arithmetic that does not overflow is unchanged,
// and the first that does panics as Rust's debug build does.
struct Account {
    balance: i32,
    history: Vec<i32>,
}

impl Account {
    fn deposit(&mut self, amount: i32) {
        self.balance += amount;
        self.history.push(amount);
    }
}

fn factorial(n: i32) -> i32 {
    (1..=n).fold(1, |acc, k| acc * k)
}

fn main() {
    let mut a = Account { balance: 0, history: vec![] };
    for k in 1..=5 {
        a.deposit(k * 100);
    }
    let n = a.history.len();
    let last = a.history[n - 1] - a.history[0];
    println!("{} {} {}", a.balance, last, -a.balance);
    let small: u8 = 200;
    println!("{}", small / 2 + 27);
    for k in [5, 10, 12] {
        println!("{k}! = {}", factorial(k));
    }
    println!("{}", factorial(13));
}
